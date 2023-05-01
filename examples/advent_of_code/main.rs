mod not_quite_lisp;
mod i_was_told_there_would_be_no_math;
use not_quite_lisp::find_floor;

fn main() -> Result<(), parsely::Error> {
    let input = "(()))()(((()())()()((()()()()()))()())(()()))))(()()()(((())()";

    println!("floor {}", find_floor(input)?);

    Ok(())
}
