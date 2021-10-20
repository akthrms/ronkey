# Go 言語でつくるインタプリタ in Rust

![rust](https://github.com/akthrms/ronkey/actions/workflows/rust.yml/badge.svg)

https://www.oreilly.co.jp/books/9784873118222/

```
$ cargo run
Hello akthrms! This is the Monkey programming language!
Feel free to type in commands
>> let a = 5;
>> let b = a > 3;
>> let c = a * 99;
>> if (b) { 10; } else { 1; }
10 : Integer
>> let d = if (c > 10) { 99; } else { 100; };
>> d;
99 : Integer
>> d * c * a;
245025 : Integer
```

```
>> let add = fn(a, b) { a + b };
>> let sub = fn(a, b) { a - b };
>> let applyFunc = fn(a, b, func) { func(a, b) };
>> applyFunc(2, 2, add);
4 : Integer
>> applyFunc(10, 2, sub);
8 : Integer
```
