# sicp-lisp

手習いに [SICP 4.1](https://mitpress.mit.edu/sites/default/files/sicp/full-text/book/book-Z-H-26.html#%_sec_4.1) を Rust で実装。
Rust が下手なので `Rc` と `RefCell` 祭り。

```
rust/sicp-lisp% rustup default nightly # for box pattern
rust/sicp-lisp% cargo run
...

> 42
42

> (define square (lambda (x) (* x x)))
ok

> (display (square 10))
100
nil

> (define make-counter (lambda () (begin (define count 0) (lambda () (begin (set! count (+ count 1)) count)))))
ok
> (define c1 (make-counter))
ok
> (define c2 (make-counter))
ok 
> (c1)
1
> (c1)
2
> (c2)
1

> (define tarai (lambda (x y z) (if (<= x y) y (tarai (tarai (- x 1) y z) (tarai (- y 1) z x) (tarai (- z 1) x y)))))
ok
> (tarai 6 3 0)
6
```
