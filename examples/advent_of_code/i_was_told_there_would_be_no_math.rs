use std::str::FromStr;

use parsely::{char, end, result_ext::*, uint, Lex, Parse};

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub struct Dimensions {
    length: usize,
    width: usize,
    height: usize,
}

impl FromStr for Dimensions {
    // this is a bit lazy of us, you can map `parsely::Error` to your own error type quite easily - we'll make a proper error handling example soon
    type Err = parsely::ErrorOwned;

    // Parsers are defined and used here
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dimensions = uint::<usize>()
            .then_skip(char('x').pad()) // note: .pad() isn't needed here since the input is never padded - that's AoC being kind, but we can handle whitespace easily!
            .then(uint::<usize>())
            .then_skip(char('x').pad())
            .then(uint::<usize>());

        // parsely isn't fancy enough to provide macros to avoid the nested tuples from repeated `.then()`s
        let (((length, width), height), _) = dimensions.then(end()).parse(s).offset(s)?;

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
    fn parse_dimensions() -> Result<(), parsely::ErrorOwned> {
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

        // too many dimensions! thanks end() :)
        let error = "10x20x30x40".parse::<Dimensions>().unwrap_err();

        assert_eq!(error.matched(), "10x20x30");
        assert_eq!(&error.remaining, "x40");

        assert_eq!(
            Dimensions {
                length: 10,
                width: 20,
                height: 30
            },
            // .pad() for the win!
            "10 x 20 x 30".parse()?
        );

        // no decimals allowed!
        let error = "10.2 x 20 x 30".parse::<Dimensions>().unwrap_err();

        assert_eq!(error.matched(), "10");
        assert_eq!(&error.remaining, ".2 x 20 x 30");

        // no leading zeroes allowed!
        let error = "001x002x003".parse::<Dimensions>().unwrap_err();

        assert_eq!(error.matched(), "0");
        assert_eq!(&error.remaining, "01x002x003");

        Ok(())
    }
}
