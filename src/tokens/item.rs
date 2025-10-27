//! A single element

use core::fmt;

use crate::lang::Lang;
use crate::tokens::{FormatInto, ItemStr, Tokens};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Kind {
    /// A literal item.
    /// Is added as a raw string to the stream of tokens.
    Literal(ItemStr),
    /// A language-specific item.
    Lang(usize),
    /// Push a new line unless the current line is empty. Will be flushed on
    /// indentation changes.
    Push,
    /// Push a line. Will be flushed on indentation changes.
    Line,
    /// Space between language items. Typically a single space.
    ///
    /// Multiple spacings in sequence are collapsed into one.
    /// A spacing does nothing if at the beginning of a line.
    Space,
    /// Manage indentation.
    ///
    /// An indentation of 0 has no effect.
    Indentation(i16),
    /// Switch to handling input as a quote.
    OpenQuote(bool),
    /// Close the last quote.
    CloseQuote,
    /// Switch on evaluation. Only valid during string handling.
    OpenEval,
    /// Close evaluation.
    CloseEval,
}

/// A single item in a stream of tokens.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Item {
    pub(crate) kind: Kind,
}

impl Item {
    /// Construct a new item based on a kind.
    #[inline]
    pub(crate) const fn new(kind: Kind) -> Self {
        Self { kind }
    }

    /// Shorthand for constructing a new static literal item.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::{Item, ItemStr};
    /// use genco::lang::Rust;
    ///
    /// let a = Item::static_("hello");
    /// let b = Item::literal("hello".into());
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn static_(literal: &'static str) -> Self {
        Self::new(Kind::Literal(ItemStr::static_(literal)))
    }

    /// Construct a new push item.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::Item;
    /// use genco::lang::Rust;
    ///
    /// let a = Item::push();
    /// let b = Item::push();
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn push() -> Self {
        Self::new(Kind::Push)
    }

    /// Construct a new line item.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::Item;
    /// use genco::lang::Rust;
    ///
    /// let a = Item::line();
    /// let b = Item::line();
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn line() -> Self {
        Self::new(Kind::Line)
    }

    /// Construct a new space item.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::Item;
    /// use genco::lang::Rust;
    ///
    /// let a = Item::space();
    /// let b = Item::space();
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn space() -> Self {
        Self::new(Kind::Space)
    }

    /// Construct a new indentation item.
    #[inline]
    pub(crate) const fn indentation(n: i16) -> Self {
        Self::new(Kind::Indentation(n))
    }

    /// Construct a quote open.
    ///
    /// Switches to handling input as a quote. The argument indicates whether
    /// the string contains any interpolated values. The string content is
    /// quoted with the language-specific [quoting method].
    ///
    /// [quoting method]: Lang::open_quote
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::Item;
    /// use genco::lang::Rust;
    ///
    /// let a = Item::open_quote(true);
    /// let b = Item::open_quote(false);
    ///
    /// assert_eq!(a, a);
    /// assert_ne!(a, b);
    /// ```
    #[inline]
    pub const fn open_quote(is_interpolated: bool) -> Self {
        Self::new(Kind::OpenQuote(is_interpolated))
    }

    /// Construct a quote close.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::Item;
    /// use genco::lang::Rust;
    ///
    /// let a = Item::close_quote();
    /// let b = Item::close_quote();
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn close_quote() -> Self {
        Self::new(Kind::CloseQuote)
    }

    /// Construct a new eval open.
    #[inline]
    pub(crate) const fn open_eval() -> Self {
        Self::new(Kind::OpenEval)
    }

    /// Construct a new eval close.
    #[inline]
    pub(crate) const fn close_eval() -> Self {
        Self::new(Kind::CloseEval)
    }

    /// Construct a new literal item.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::{Item, ItemStr};
    /// use genco::lang::Rust;
    ///
    /// let a = Item::static_("hello");
    /// let b = Item::literal("hello".into());
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn literal(lit: ItemStr) -> Self {
        Self::new(Kind::Literal(lit))
    }

    /// Construct a lang item with the given index.
    #[inline]
    pub(crate) const fn lang(index: usize) -> Item {
        Item::new(Kind::Lang(index))
    }
}

impl fmt::Debug for Item {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

/// Formatting an item is the same as simply adding that item to the token
/// stream.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
/// use genco::tokens::{Item, ItemStr};
///
/// let foo = Item::literal(ItemStr::static_("foo"));
/// let bar = Item::literal("bar".into());
///
/// let result: Tokens = quote!($foo $bar baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
///
/// assert_eq!{
///     vec![
///         Item::literal(ItemStr::static_("foo")),
///         Item::space(),
///         Item::literal("bar".into()),
///         Item::space(),
///         Item::literal(ItemStr::static_("baz")),
///     ],
///     result,
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for Item
where
    L: Lang,
{
    #[inline]
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(self);
    }
}
