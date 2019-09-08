use super::frontend::*;

use cranelift::prelude::*;
use cranelift_module::{default_libcall_names, DataContext, Linkage, Module};
use cranelift_simplejit::{SimpleJITBackend, SimpleJITBuilder};

const DATA_SIZE: i32 = 65535;

pub struct JIT {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    data_ctx: DataContext,
    module: Module<SimpleJITBackend>,
}

impl JIT {
    pub fn new() -> Self {
        if cfg!(windows) {
            unimplemented!();
        }

        let builder = SimpleJITBuilder::new(default_libcall_names());
        let module = Module::new(builder);
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_ctx: DataContext::new(),
            module,
        }
    }

    pub fn compile(&mut self, input: &str) -> Result<*const u8, String> {
        let commands = parser::program(&input).unwrap();
        self.initialize_memory();
        self.translate(&commands).unwrap();

        let main = self
            .module
            .declare_function("main", Linkage::Export, &self.ctx.func.signature)
            .unwrap();
        self.module.define_function(main, &mut self.ctx).unwrap();
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions();
        let code = self.module.get_finalized_function(main);
        Ok(code)
    }

    fn initialize_memory(&mut self) {
        self.data_ctx.define_zeroinit(4 * DATA_SIZE as usize);
        let id = self
            .module
            .declare_data("data", Linkage::Export, true, None)
            .unwrap();
        self.module.define_data(id, &self.data_ctx).unwrap();
        self.data_ctx.clear();
        self.module.finalize_definitions();
    }

    fn translate(&mut self, commands: &[Expr]) -> Result<(), String> {
        let pointer_type = self.module.target_config().pointer_type();

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let entry_ebb = builder.create_ebb();
        builder.switch_to_block(entry_ebb);
        builder.seal_block(entry_ebb);

        let getchar = {
            let mut sig = self.module.make_signature();
            sig.returns.push(AbiParam::new(types::I32));
            let callee = self
                .module
                .declare_function("getchar", Linkage::Import, &sig)
                .unwrap();
            self.module.declare_func_in_func(callee, &mut builder.func)
        };

        let putchar = {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I32));
            sig.returns.push(AbiParam::new(types::I32));
            let callee = self
                .module
                .declare_function("putchar", Linkage::Import, &sig)
                .unwrap();
            self.module.declare_func_in_func(callee, &mut builder.func)
        };

        let data = {
            let sym = self
                .module
                .declare_data("data", Linkage::Export, true, None)
                .unwrap();
            let id = self.module.declare_data_in_func(sym, &mut builder.func);
            builder.ins().symbol_value(pointer_type, id)
        };

        let zero = builder.ins().iconst(pointer_type, 0);
        let ptr = Variable::new(0);
        builder.declare_var(ptr, pointer_type);
        builder.def_var(ptr, zero);

        let mut translator = FunctionTranslator {
            pointer_type,
            builder,
            getchar,
            putchar,
            ptr,
            data,
        };
        translator.translate(commands);

        translator.builder.ins().return_(&[]);
        translator.builder.finalize();
        Ok(())
    }
}

struct FunctionTranslator<'a> {
    pointer_type: types::Type,
    builder: FunctionBuilder<'a>,
    getchar: codegen::ir::entities::FuncRef,
    putchar: codegen::ir::entities::FuncRef,
    ptr: Variable,
    data: Value,
}

impl<'a> FunctionTranslator<'a> {
    fn translate(&mut self, commands: &[Expr]) {
        for expr in commands {
            match expr {
                Expr::Add(count) => {
                    let p = self.builder.use_var(self.ptr);
                    let p = self.builder.ins().iadd(self.data, p);
                    let v = self.builder.ins().load(types::I32, MemFlags::new(), p, 0);
                    let c = self.builder.ins().iconst(types::I32, i64::from(*count));
                    let s = self.builder.ins().iadd(v, c);
                    self.builder.ins().store(MemFlags::new(), s, p, 0);
                }
                Expr::Sub(count) => {
                    let p = self.builder.use_var(self.ptr);
                    let p = self.builder.ins().iadd(self.data, p);
                    let v = self.builder.ins().load(types::I32, MemFlags::new(), p, 0);
                    let c = self.builder.ins().iconst(types::I32, i64::from(*count));
                    let s = self.builder.ins().isub(v, c);
                    self.builder.ins().store(MemFlags::new(), s, p, 0);
                }
                Expr::Right(offset) => {
                    let p = self.builder.use_var(self.ptr);
                    let c = self
                        .builder
                        .ins()
                        .iconst(self.pointer_type, 4 * (*offset as i64));
                    let v = self.builder.ins().iadd(p, c);
                    self.builder.def_var(self.ptr, v);
                }
                Expr::Left(offset) => {
                    let p = self.builder.use_var(self.ptr);
                    let c = self
                        .builder
                        .ins()
                        .iconst(self.pointer_type, 4 * (*offset as i64));
                    let v = self.builder.ins().isub(p, c);
                    self.builder.def_var(self.ptr, v);
                }
                Expr::Clear => {
                    let p = self.builder.use_var(self.ptr);
                    let p = self.builder.ins().iadd(self.data, p);
                    let zero = self.builder.ins().iconst(types::I32, 0);
                    self.builder.ins().store(MemFlags::new(), zero, p, 0);
                }
                Expr::Out => {
                    let p = self.builder.use_var(self.ptr);
                    let p = self.builder.ins().iadd(self.data, p);
                    let v = self.builder.ins().load(types::I32, MemFlags::new(), p, 0);
                    self.builder.ins().call(self.putchar, &[v]);
                }
                Expr::In => {
                    let call = self.builder.ins().call(self.getchar, &[]);
                    let result = self.builder.inst_results(call)[0];

                    let p = self.builder.use_var(self.ptr);
                    let p = self.builder.ins().iadd(self.data, p);
                    self.builder.ins().store(MemFlags::new(), result, p, 0);
                }
                Expr::Loop(commands) => {
                    let header_block = self.builder.create_ebb();
                    let exit_block = self.builder.create_ebb();
                    self.builder.ins().jump(header_block, &[]);
                    self.builder.switch_to_block(header_block);

                    let p = self.builder.use_var(self.ptr);
                    let p = self.builder.ins().iadd(self.data, p);
                    let flag = self.builder.ins().load(types::I32, MemFlags::new(), p, 0);
                    self.builder.ins().brz(flag, exit_block, &[]);

                    self.translate(commands);
                    self.builder.ins().jump(header_block, &[]);

                    self.builder.switch_to_block(exit_block);

                    self.builder.seal_block(header_block);
                    self.builder.seal_block(exit_block);
                }
            }
        }
    }
}
