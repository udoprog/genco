//! Helper trait to take ownership of strings.

use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;

#[cfg(feature = "alloc")]
use alloc::borrow::{Cow, ToOwned};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::rc::Rc;
#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::lang::Lang;
use crate::tokens::{FormatInto, Item, Tokens};

/// Internal representation.
#[derive(Clone)]
enum ItemStrKind {
    /// A boxed string.
    #[cfg(feature = "alloc")]
    Box(Box<str>),
    /// A static string.
    Static(&'static str),
}

/// A managed string that permits immutable borrowing.
#[derive(Clone)]
pub struct ItemStr {
    kind: ItemStrKind,
}

impl ItemStr {
    /// Create a new ItemStr from the given kind.
    #[inline]
    const fn new(kind: ItemStrKind) -> Self {
        Self { kind }
    }

    /// Construct a new string item based on a static string.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::ItemStr;
    ///
    /// let string = ItemStr::static_("hello world");
    /// assert_eq!(&string[..], "hello world");
    /// ```
    pub const fn static_(s: &'static str) -> Self {
        Self::new(ItemStrKind::Static(s))
    }
}

/// Convert stringy things.
impl<L> FormatInto<L> for ItemStr
where
    L: Lang,
{
    #[inline]
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.append(Item::Literal(self));
    }
}

impl<L> FormatInto<L> for &ItemStr
where
    L: Lang,
{
    #[inline]
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.append(Item::Literal(self.clone()));
    }
}

impl AsRef<str> for ItemStr {
    #[inline]
    fn as_ref(&self) -> &str {
        match &self.kind {
            #[cfg(feature = "alloc")]
            ItemStrKind::Box(b) => b,
            ItemStrKind::Static(s) => s,
        }
    }
}

impl Deref for ItemStr {
    type Target = str;

    fn deref(&self) -> &str {
        match &self.kind {
            #[cfg(feature = "alloc")]
            ItemStrKind::Box(b) => b,
            ItemStrKind::Static(s) => s,
        }
    }
}

#[cfg(feature = "alloc")]
impl From<Box<str>> for ItemStr {
    #[inline]
    fn from(value: Box<str>) -> Self {
        Self::new(ItemStrKind::Box(value))
    }
}

impl<'a> From<&'a ItemStr> for ItemStr {
    #[inline]
    fn from(value: &'a ItemStr) -> Self {
        value.clone()
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a String> for ItemStr {
    #[inline]
    fn from(value: &'a String) -> Self {
        value.clone().into()
    }
}

#[cfg(feature = "alloc")]
impl From<String> for ItemStr {
    #[inline]
    fn from(value: String) -> Self {
        Self::new(ItemStrKind::Box(value.into_boxed_str()))
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a str> for ItemStr {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::new(ItemStrKind::Box(value.to_owned().into_boxed_str()))
    }
}

#[cfg(feature = "alloc")]
impl<'a, 'b> From<&'b &'a str> for ItemStr {
    #[inline]
    fn from(value: &'b &'a str) -> Self {
        Self::new(ItemStrKind::Box((*value).to_owned().into_boxed_str()))
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<Cow<'a, str>> for ItemStr {
    #[inline]
    fn from(value: Cow<'a, str>) -> Self {
        Self::new(ItemStrKind::Box(match value {
            Cow::Owned(string) => string.into_boxed_str(),
            Cow::Borrowed(string) => string.to_owned().into_boxed_str(),
        }))
    }
}

#[cfg(feature = "alloc")]
impl<'a, 'b> From<&'b Cow<'a, str>> for ItemStr {
    #[inline]
    fn from(value: &'b Cow<'a, str>) -> Self {
        Self::new(ItemStrKind::Box(match value {
            Cow::Owned(string) => string.clone().into_boxed_str(),
            Cow::Borrowed(string) => (*string).to_owned().into_boxed_str(),
        }))
    }
}

#[cfg(feature = "alloc")]
impl From<Rc<String>> for ItemStr {
    #[inline]
    fn from(value: Rc<String>) -> Self {
        Self::new(ItemStrKind::Box((*value).clone().into()))
    }
}

impl fmt::Debug for ItemStr {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(fmt)
    }
}

impl fmt::Display for ItemStr {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(fmt)
    }
}

impl PartialEq for ItemStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for ItemStr {}

impl PartialOrd for ItemStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.as_ref().cmp(other.as_ref()))
    }
}

impl Ord for ItemStr {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl Hash for ItemStr {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_ref().hash(state);
    }
}
