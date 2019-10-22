# Conway's Game of Life

Rust, WebAssembly, WebXR

## Movie

[![](https://img.youtube.com/vi/cRH0_hcnbgA/0.jpg)](https://www.youtube.com/watch?v=cRH0_hcnbgA)

## Demo

https://seikichi.github.io/sandbox/rust/wasm-3d-game-of-life (or https://bit.ly/2qDcXqD)

Also, please check [supported browsers](https://github.com/mozilla/webxr-polyfill#supported-browsers).

## Build

### wasm

```
> wasm-pack build # generate pkg/*
> wasm-opt -Oz --dce -o output.wasm pkg/wasm_3d_game_of_life_bg.wasm
> mv output.wasm pkg/wasm_3d_game_of_life_bg.wasm
```

### web

```
> cd www
> npm i
> npm run build # generate dist/*
```
