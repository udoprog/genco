//! Helper container for borrowed or owned values.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Contained<'a, T: 'a> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T> AsRef<T> for Contained<'a, T> {
    fn as_ref(&self) -> &T {
        use self::Contained::*;

        match *self {
            Borrowed(value) => value,
            Owned(ref value) => value,
        }
    }
}
