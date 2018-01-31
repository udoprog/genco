//! Helper trait to take ownership of strings.

use std::rc::Rc;
use std::ops::Deref;
use std::borrow::Cow;

/// A managed string that permits immutable borrowing.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Cons<'el> {
    /// A borrowed string.
    Borrowed(&'el str),
    /// A refcounted string.
    Rc(Rc<String>),
}

impl<'a> AsRef<str> for Cons<'a> {
    fn as_ref(&self) -> &str {
        use self::Cons::*;

        match *self {
            Borrowed(value) => value,
            Rc(ref value) => value.as_ref(),
        }
    }
}

impl<'a> Deref for Cons<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_ref()
    }
}

impl<'el> From<String> for Cons<'el> {
    fn from(value: String) -> Self {
        Cons::Rc(Rc::new(value))
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

impl<'el> From<Cow<'el, str>> for Cons<'el> {
    fn from(value: Cow<'el, str>) -> Self {
        use self::Cow::*;

        match value {
            Owned(string) => Cons::Rc(Rc::new(string)),
            Borrowed(string) => Cons::Borrowed(string),
        }
    }
}
