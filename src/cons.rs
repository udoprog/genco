//! Helper trait to take ownership of strings.

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cons<'element> {
    Borrowed(&'element str),
    Owned(String),
    Rc(Rc<String>),
}

impl<'a> AsRef<str> for Cons<'a> {
    fn as_ref(&self) -> &str {
        use self::Cons::*;

        match *self {
            Borrowed(value) => value,
            Owned(ref value) => value,
            Rc(ref value) => value.as_ref(),
        }
    }
}
