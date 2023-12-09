use parsely::{char, int, Parse};

pub fn csv<P: Parse>(item: P) -> impl Parse<Output = Vec<<P as Parse>::Output>> {
    item.pad().many(..).delimiter(char(','))
}

pub fn main() -> Result<(), parsely::Error> {
    let (output, remaining) = csv(int::<u8>()).parse("1,2,3, 4 , 5")?;

    println!("numbers sum up to {}", output.iter().sum::<u8>());
    assert_eq!(remaining, "");

    // Note that the following is a compile error, because digit() is a Lexer, not a Parser
    // let (output, remaining) = csv(digit()).parse("1,2,3, 4 , 5")?;
    // ```
    // expected a `std::ops::Fn<(&str,)>` closure, found `parsely::Digit`
    // the trait `for<'a> std::ops::Fn<(&'a str,)>` is not implemented for `parsely::Digit`
    // the following other types implement trait `parsely::Parse`:
    // ```

    // The error above is confusing because Rust complains about a Fn trait - when really it's the Parse trait we're interested in.
    // This happens because Parse is blanket implemented for functions that take &str (and return a ParseResult).
    // Digit doesn't impl Parse, and it isn't a function that implements Parse.
    // This error is Rust's way of "helpfully" pointing out that *if* Digit was a function that implemented Parse, then Digit would implement Parse.

    Ok(())
}
