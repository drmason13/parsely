//! The built in combinators provided by parsely

// Combinator TODO list:
// * then_with -> <https://docs.rs/chumsky/latest/chumsky/trait.Parser.html#method.then_with>

mod map;
mod optional;
mod or;
pub mod sequence;
mod skip;
mod then;

#[doc(inline)]
pub use self::sequence::{count, delimited, many, Delimited, Many};

#[doc(inline)]
pub use self::map::{map, try_map, Map, TryMap};

#[doc(inline)]
pub use self::optional::{optional, Optional};

#[doc(inline)]
pub use self::or::{or, Or};

#[doc(inline)]
pub use self::skip::{skip_then, then_skip, SkipThen, ThenSkip};

#[doc(inline)]
pub use self::then::{then, Then};
