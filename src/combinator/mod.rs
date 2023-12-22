//! The built in combinators provided by parsely

mod crawl;
mod map;
mod optional;
mod or;
mod pad;
pub mod sequence;
pub mod skip;
mod then;

#[doc(inline)]
pub use self::crawl::{crawl, Crawl};
#[doc(inline)]
pub use self::map::{map, try_map, Map, TryMap};
#[doc(inline)]
pub use self::optional::{optional, Optional};
#[doc(inline)]
pub use self::or::{or, Or};
#[doc(inline)]
pub use self::pad::{pad, Pad};
#[doc(inline)]
pub use self::sequence::{all, count, delimited, many, traits::Collect, All, Delimited, Many};
#[doc(inline)]
pub use self::skip::{skip_then, then_skip, SkipThen, ThenSkip};
#[doc(inline)]
pub use self::then::{then, Then};
