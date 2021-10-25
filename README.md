# Go 言語でつくるインタプリタ in Rust

![rust](https://github.com/akthrms/ronkey/actions/workflows/rust.yml/badge.svg)

https://www.oreilly.co.jp/books/9784873118222/

```
$ cargo run
Hello akthrms! This is the Monkey programming language!
Feel free to type in commands
>> let a = 1
>> let b = 2
>> let add = fn(x, y) { return x + y }
>> add(a, b)
3
>> if ("a" == "A") { "eq" } else { "ne" }
ne
>> let c = ["a", "b", "c"]
>> c[1]
b
>> let d = {"name": "akthrms", "country": "japan"}
>> d["name"]
akthrms
```
