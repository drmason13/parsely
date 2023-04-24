//! Combinator TODO list:
//! * then_with -> https://docs.rs/chumsky/latest/chumsky/trait.Parser.html#method.then_with

pub mod many;
pub mod map;
pub mod optional;
pub mod or;
pub mod skip;
pub mod then;

#[doc(inline)]
pub use many::{count, many, Many};

#[doc(inline)]
pub use map::{map, try_map, Map, TryMap};

#[doc(inline)]
pub use optional::{optional, Optional};

#[doc(inline)]
pub use or::{or, Or};

#[doc(inline)]
pub use skip::{skip_then, then_skip, SkipThen, ThenSkip};

#[doc(inline)]
pub use then::{then, Then};
