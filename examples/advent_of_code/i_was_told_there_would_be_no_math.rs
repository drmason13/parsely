use std::str::FromStr;

use parsely::{char, end, uint, Lex, Parse};

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub struct Dimensions {
    length: usize,
    width: usize,
    height: usize,
}

impl FromStr for Dimensions {
    // this is a bit lazy of us, you can map `parsely::Error` to your own error type quite easily - we'll make a proper error handling example soon
    type Err = parsely::Error;

    // Parsers are defined and used here
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dimensions = uint::<usize>()
            .then_skip(char('x').pad()) // note: .pad() isn't needed here since the input is never padded - that's AoC being kind, but we can handle whitespace easily!
            .then(uint::<usize>())
            .then_skip(char('x').pad())
            .then(uint::<usize>());

        // parsely isn't fancy enough to provide macros to avoid the nested tuples from repeated `.then()`s
        let (((length, width), height), _) = dimensions.then(end()).parse(s)?;

        Ok(Dimensions {
            length,
            width,
            height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dimensions() -> Result<(), parsely::Error> {
        assert_eq!(
            "1x2x3".parse::<Dimensions>()?,
            Dimensions {
                length: 1,
                width: 2,
                height: 3
            }
        );

        assert_eq!(
            Dimensions {
                length: 10,
                width: 20,
                height: 30
            },
            "10x20x30".parse()?
        );

        assert_eq!(
            Err(parsely::Error::NoMatch),
            "10x20x30x40".parse::<Dimensions>() // too many dimensions! thanks end() :)
        );

        assert_eq!(
            Dimensions {
                length: 10,
                width: 20,
                height: 30
            },
            "10 x 20 x 30".parse()? // .pad() for the win!
        );

        assert_eq!(
            Err(parsely::Error::NoMatch),
            "10.2 x 20 x 30".parse::<Dimensions>() // no decimals allowed!
        );

        assert_eq!(
            Err(parsely::Error::NoMatch),
            "001x002x003".parse::<Dimensions>()
        );

        Ok(())
    }
}
