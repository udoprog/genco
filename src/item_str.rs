//! Helper trait to take ownership of strings.

use crate::{FormatTokens, Lang, Tokens};
use std::borrow::Cow;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

/// A managed string that permits immutable borrowing.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum ItemStr {
    /// A refcounted string.
    Box(Box<str>),
    /// A static string.
    Static(&'static str),
}

/// Convert stringy things.
impl<L> FormatTokens<L> for ItemStr
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.push_item(self.into());
    }
}

impl<'a, L> FormatTokens<L> for &'a ItemStr
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.push_item(self.clone().into());
    }
}

impl AsRef<str> for ItemStr {
    fn as_ref(&self) -> &str {
        match self {
            Self::Box(rc) => &**rc,
            Self::Static(s) => s,
        }
    }
}

impl Deref for ItemStr {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            Self::Box(rc) => &**rc,
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
        Self::Box(value.to_string().into_boxed_str())
    }
}

impl From<Rc<String>> for ItemStr {
    fn from(value: Rc<String>) -> Self {
        Self::Box(value.to_string().into_boxed_str())
    }
}

impl<'a> From<Cow<'a, str>> for ItemStr {
    fn from(value: Cow<'a, str>) -> Self {
        Self::Box(match value {
            Cow::Owned(string) => string.into_boxed_str(),
            Cow::Borrowed(string) => string.to_string().into_boxed_str(),
        })
    }
}

impl<'a, 'b> From<&'a Cow<'b, str>> for ItemStr {
    fn from(value: &'a Cow<'b, str>) -> Self {
        Self::Box(match value {
            Cow::Owned(string) => string.to_string().into_boxed_str(),
            Cow::Borrowed(string) => string.to_string().into_boxed_str(),
        })
    }
}

impl fmt::Display for ItemStr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(fmt)
    }
}
