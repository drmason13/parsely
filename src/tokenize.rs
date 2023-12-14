//! [`Tokenize`] allows us to output "Tokens" that borrow from the input &str
//!
//! Any lexer can be used to build a tokenizer
//!
//! A tokenizer is a parser that is able to borrow from the input str.
//! This involves some additional lifetime complexity but allows parser to avoid unnecessary allocations!
//!

// fails to compile
// pub trait Tokenize {
//     type Token<'i>: 'i;

//     fn tokenize(&self, input: &'i str) -> Result<(Self::Token<'i>, &'i str), crate::Error>;
// }

use crate::{token, Lex, Parse, ParseResult};

pub struct Token<'i>(&'i str);

fn parse_token1<'i>(input: &'i str) -> Token<'i> {
    let (token, _) = "foo".lex(input).unwrap();
    Token(token)
}

fn make_token<'i>(input: &'i str) -> Token<'i> {
    Token(input)
}

fn parse_token<'i>(input: &'i str) -> Token<'i> {
    let (token, _) = "foo".lex(input).unwrap();
    make_token(token)
}

fn parser<'i>(input: &'i str) -> ParseResult<'i, Token<'i>> {
    let token = parse_token(input);
    let (_, remaining) = "foo".lex(input).unwrap();
    Ok((token, remaining))
}

// fails to compile: There's no way to make
// ```
// for<'i> fn(&'i str) -> std::result::Result<(tokenize::Token<'i>, &'i str)
// ```
// `impl Parse<Output = Token<'i>>` because there's no specific 'i to mention!
//
// fn combinate() -> impl Parse<Output = Vec<Token<'i>>> {
//     "token".skip_then(parser)
// }

trait Tokenable {
    type Token<'a>: TokenMarker<'a>
    where
        Self: 'a;

    fn tokenize(&self) -> Self::Token<'_>;
}

impl<'i> Tokenable for &'i str {
    type Token<'t> = Token<'t>
    where Self: 't;

    fn tokenize(&self) -> Self::Token<'_> {
        parse_token1(self)
    }
}

pub trait TokenMarker<'a> {}

impl<'a> TokenMarker<'a> for Token<'a> {
    // no idae what this is doing really
}

fn tokenize_maybe(input: &str) -> Token<'_> {
    // the below fails to compile
    //     input.tokenize()
    // with
    //     error[E0515]: cannot return value referencing function parameter `input`
    //     --> src/tokenize.rs:75:5
    //      |
    //   75 |     input.tokenize()
    //      |     -----^^^^^^^^^^^
    //      |     |
    //      |     returns a value referencing data owned by the current function
    //      |     `input` is borrowed here

    todo!()
}
