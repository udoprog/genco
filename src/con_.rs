//! Helper container for borrowed or owned values.

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Con<'a, T: 'a> {
    Borrowed(&'a T),
    Owned(T),
    Rc(Rc<T>),
}

impl<'a, T> AsRef<T> for Con<'a, T> {
    fn as_ref(&self) -> &T {
        use self::Con::*;

        match *self {
            Borrowed(value) => value,
            Owned(ref value) => value,
            Rc(ref value) => value.as_ref(),
        }
    }
}
