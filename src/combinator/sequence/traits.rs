//! The sequence traits abstract how parsely sequence combinators repeatedly apply a lexer or parser to an input
//!
//! These traits should not need to be implemented manually, prefer to use existing combinators such as [`many()`](crate::combinator::many)
use std::ops::ControlFlow;

use crate::{Error, Lex, Parse};

/// Describes how a sequence combinator behaves while processing input
pub trait Sequence: Collect {
    /// The sequencer continues to process input **while this returns true**
    fn while_condition(&self, input: &str, count: usize) -> bool;

    /// The sequencer returns an error instead of succeeding if this returns true
    ///
    /// It is called after all processable input has been processed
    fn error_condition(&self, input: &str, count: usize) -> bool;
}

/// All sequence combinators must provide a way to change the collection type they use to store output
///
/// The `collect` method should be implemented directly on the type, so that users can use it directly without importing this trait
pub trait Collect {
    /// The type returned when calling collect, where C is the new Collection type to use
    ///
    /// Almost always `Self<C>` but we have to use an associated type to describe that
    type Output<C>;

    /// Change the collection used by a [sequencer](Sequence) to C
    fn collect<C1>(self) -> Self::Output<C1>
    where
        Self: Sized;
    // Self::Output<C1>: ParseSequence<C1>;
}

/// All sequence combinators impl both [`LexSequence`] and [`ParseSequence`]
pub trait LexSequence: Sequence {
    /// The [`Lexer`](crate::Lex) to apply repeatedly
    type Lexer: Lex;

    /// progress through one iteration of lexing
    fn lex_one<'i>(
        &self,
        input: &'i str,
        working_input: &mut &'i str,
        count: &mut usize,
        offset: &mut usize,
        error: &mut Option<Error<'i>>,
    ) -> ControlFlow<(), &'i str>;
}

/// All sequence combinators impl both [`LexSequence`] and [`ParseSequence`]
pub trait ParseSequence<C>: Sequence
where
    C: Extend<<Self::Parser as Parse>::Output>,
{
    /// The [`Parser`](crate::Parse) to apply repeatedly
    type Parser: Parse;

    /// progress through one iteration of parsing
    fn parse_one<'i>(
        &self,
        input: &'i str,
        working_input: &mut &'i str,
        count: &mut usize,
        offset: &mut usize,
        error: &mut Option<Error<'i>>,
        outputs: &mut C,
    ) -> ControlFlow<(), &'i str>;
}
