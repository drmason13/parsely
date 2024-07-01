# Parsely 🌿

Parsely is a simple string parsing library for Rust with the following aims

* Excel when used to `impl FromStr` for your types
* Simple to use
* Well documented

Note: This crate isn't published to crates.io yet, I'm still working on an initial release. I tend to add combinators as and when I encounter the need for them.

## Example

```rust
use parsely::{ch_if, Lex, Parse, ParseResult};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl FromStr for Color {
    type Err = parsely::ErrorOwned;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(hex_color(s)?.0)
    }
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn hex_primary() -> impl Parse<Output = u8> {
    ch_if(is_hex_digit).count(2).try_map(from_hex)
}

fn hex_color(input: &str) -> ParseResult<Color> {
    let (((red, green), blue), remaining) = "#"
        .skip_then(hex_primary().then(hex_primary()).then(hex_primary()))
        .parse(input)?;

    Ok((Color { red, green, blue }, remaining))
}

fn main() {
    assert_eq!(
        "#2F14DF".parse::<Color>(),
        Ok(Color {
            red: 47,
            green: 20,
            blue: 223,
        })
    );
}
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>