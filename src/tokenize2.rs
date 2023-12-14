/// I tried to make my code generic over some sort of "lockable" types. Like a Mutex<T>,
/// which you can call .lock() on, and after you locked it there is a set of methods
/// you can call on T.
///
/// I didn't want to lock the user into a predetermined Mutex type, so I came up with this trait:
trait Tokenable {
    type Token<'a>: TokenMarker<'a>
    where
        Self: 'a;
    fn tokenize(&self) -> Self::Token<'_>;
}

pub struct Token<'i>(&'i str);

impl<'i> Tokenable for &'i str {
    type Token<'t> = Token<'t>
    where Self: 't;

    fn tokenize(&self) -> Self::Token<'_> {
        todo!()
    }
}

pub trait TokenMarker<'a> {}

impl<'a> TokenMarker<'a> for Token<'a> {}
