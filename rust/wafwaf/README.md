# wafwaf

社内の読書会で...

- [tokio](https://github.com/tokio-rs/tokio) や [hyper](https://github.com/hyperium/hyper) といったフレームワークを利用し Sinatra や Express っぽい WAF を作り
- 作った WAF を用いて以下を実装
  - カウンター付きホームページ (「あなたは X 人目のお客様です！」)
  - [JSON Serialization](https://github.com/TechEmpower/FrameworkBenchmarks/wiki/Project-Information-Framework-Tests-Overview#json-serialization)

をやることになったやつ (**なぜ...**)。

## Hello, world!

```sh
> cargo run --example hello &
> curl localhost:8080/hello
Hello, world!

> curl localhost:8080/hello/seikichi
Hello, seikichi!
```

## Counter

```sh
> cargo run --example counter &
> curl localhost:8080

<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Welcome</title>
  </head>
  <body>
    <p>You are 1th guest.</p>
  </body>
</html>
```

## JSON

```sh
> cargo run --example json &
> curl localhost:8080/json
{"message":"Hello, world!"}
```
