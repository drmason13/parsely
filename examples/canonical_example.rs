use parsely::{char_if, Lex, Parse, ParseResult};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl FromStr for Color {
    type Err = parsely::Error;

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
    char_if(is_hex_digit).count(2).try_map(from_hex)
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
