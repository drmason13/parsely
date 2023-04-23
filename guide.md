
We'll start with this example implementation of parsing an int and fix the errors, and improve it along the way.
```rust
use parsely::{char, digit, Lex, Parse};

pub fn int<T>() -> impl Parse<Output = T> {
    char('-')
        .many(0..=1)
        .then(digit().many(1..))
        .map(|n| n.parse())
}
```
This gives a nasty compiler error like:

```text
number.rs(15, 12): this type parameter
map.rs(18, 20): required for `combinator::map::Map<combinator::then::Then<combinator::many::Many<lexer::char::Char>, ...>, ...>` to implement `parse::Parse`
expected `[closure@src/parser/number.rs:19:14: 19:17]` to be a closure that returns `T`, but it returns `std::result::Result<_, _>`
    expected enum `std::result::Result<_, _>`
found type parameter `T`
the full type name has been written to '/home/masond/rust/parsely/target/debug/deps/parsely-aa0d50314c3dd08c.long-type-12960484443736369051.txt'rustc
number.rs(15, 12): this type parameter
map.rs(18, 20): required for `combinator::map::Map<combinator::then::Then<combinator::many::Many<lexer::char::Char>, ...>, ...>` to implement `parse::Parse`
```

The problem here is that we're using `.map()` which is for infallible conversions and then returning a Result from `n.parse()`.

Simple fix is to use `try_map` instead:

```rust
use parsely::{char, digit, Lex, Parse};

pub fn int<T>() -> impl Parse<Output = T> {
    char('-')
        .many(0..=1)
        .then(digit().many(1..))
        .try_map(|n| n.parse())
}
```

This still fails to compile:

```text
the trait bound `T: std::str::FromStr` is not satisfied
the trait `std::str::FromStr` is not implemented for `T`rustc
mod.rs(2350, 21): required by a bound in `core::str::<impl str>::parse`
number.rs(15, 13): consider restricting type parameter `T`: `: std::str::FromStr`
```

which makes sense, we knew that T is meant to be some integer type that converts to a string, but the compiler doesn't. Let's add the bound as suggested

```rust
use std::str::FromStr;

use parsely::{char, digit, Lex, Parse};

pub fn int<T: FromStr>() -> impl Parse<Output = T> {
    char('-')
        .many(0..=1)
        .then(digit().many(1..))
        .try_map(|n| n.parse())
}
```

We are compiling now, but if we try to use our parser, it doesn't work!

```rust
use std::str::FromStr;

use parsely::{char, digit, Lex, Parse};

pub fn int<T: FromStr>() -> impl Parse<Output = T> {
    char('-')
        .many(0..=1)
        .then(digit().many(1..))
        .try_map(|n| n.parse())
}

let result = int().parse("123");
```