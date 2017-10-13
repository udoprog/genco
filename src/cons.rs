//! Helper trait to take ownership of strings.

use std::rc::Rc;

/// A managed string that permits immutable borrowing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cons<'el> {
    /// A borrowed string.
    Borrowed(&'el str),
    /// An owned string (deprecate?)
    Owned(String),
    /// A refcounted string.
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

impl<'el> From<String> for Cons<'el> {
    fn from(value: String) -> Self {
        Cons::Owned(value)
    }
}

impl<'el> From<&'el str> for Cons<'el> {
    fn from(value: &'el str) -> Self {
        Cons::Borrowed(value)
    }
}

impl<'el> From<Rc<String>> for Cons<'el> {
    fn from(value: Rc<String>) -> Self {
        Cons::Rc(value)
    }
}
