//! Helper trait to take ownership of strings.

use core::fmt;
use core::ops::Deref;

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;

use crate::lang::Lang;
use crate::tokens::{FormatInto, Item, Tokens};

/// A managed string that permits immutable borrowing.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum ItemStr {
    /// A boxed string.
    Box(Box<str>),
    /// A static string.
    Static(&'static str),
}

/// Convert stringy things.
impl<L> FormatInto<L> for ItemStr
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.append(Item::Literal(self));
    }
}

impl<L> FormatInto<L> for &ItemStr
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.append(Item::Literal(self.clone()));
    }
}

impl AsRef<str> for ItemStr {
    fn as_ref(&self) -> &str {
        match self {
            Self::Box(b) => b,
            Self::Static(s) => s,
        }
    }
}

impl Deref for ItemStr {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            Self::Box(b) => b,
            Self::Static(s) => s,
        }
    }
}

impl From<Box<str>> for ItemStr {
    fn from(value: Box<str>) -> Self {
        Self::Box(value)
    }
}

impl<'a> From<&'a ItemStr> for ItemStr {
    fn from(value: &'a ItemStr) -> Self {
        value.clone()
    }
}

impl<'a> From<&'a String> for ItemStr {
    fn from(value: &'a String) -> Self {
        Self::Box(value.clone().into_boxed_str())
    }
}

impl From<String> for ItemStr {
    fn from(value: String) -> Self {
        Self::Box(value.into_boxed_str())
    }
}

impl<'a> From<&'a str> for ItemStr {
    fn from(value: &'a str) -> Self {
        Self::Box(value.to_owned().into_boxed_str())
    }
}

impl<'a, 'b> From<&'b &'a str> for ItemStr {
    fn from(value: &'b &'a str) -> Self {
        Self::Box((*value).to_owned().into_boxed_str())
    }
}

impl<'a> From<Cow<'a, str>> for ItemStr {
    fn from(value: Cow<'a, str>) -> Self {
        Self::Box(match value {
            Cow::Owned(string) => string.into_boxed_str(),
            Cow::Borrowed(string) => string.to_owned().into_boxed_str(),
        })
    }
}

impl<'a, 'b> From<&'b Cow<'a, str>> for ItemStr {
    fn from(value: &'b Cow<'a, str>) -> Self {
        Self::Box(match value {
            Cow::Owned(string) => string.clone().into_boxed_str(),
            Cow::Borrowed(string) => (*string).to_owned().into_boxed_str(),
        })
    }
}

impl From<Rc<String>> for ItemStr {
    fn from(value: Rc<String>) -> Self {
        Self::Box((*value).clone().into())
    }
}

impl fmt::Display for ItemStr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(fmt)
    }
}
