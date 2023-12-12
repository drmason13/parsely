mod i_was_told_there_would_be_no_math;
mod not_quite_lisp;
use not_quite_lisp::find_floor;

fn main() -> Result<(), parsely::ErrorOwned> {
    let input = "(()))()(((()())()()((()()()()()))()())(()()))))(()()()(((())()";

    println!("floor {}", find_floor(input)?);

    Ok(())
}
