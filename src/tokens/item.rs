//! A single element

use crate::lang::{Lang, LangItem};
use crate::tokens::{FormatInto, ItemStr, Tokens};
use std::cmp;

/// A single item in a stream of tokens.
pub enum Item<L>
where
    L: Lang,
{
    /// A literal item.
    /// Is added as a raw string to the stream of tokens.
    Literal(ItemStr),
    /// A language-specific boxed item.
    Lang(Box<dyn LangItem<L>>),
    /// A language-specific boxed item that is not rendered.
    Register(Box<dyn LangItem<L>>),
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
    ///
    /// The argument indicates whether the string contains any interpolated
    /// values.
    ///
    /// The string content is quoted with the language-specific [quoting method].
    /// [quoting method]: Lang::Openquote_string
    OpenQuote(bool),
    /// Close the current quote.
    CloseQuote,
    /// Switch on evaluation. Only valid during string handling.
    OpenEval,
    /// Close evaluation.
    CloseEval,
}

/// Formatting an item is the same as adding said item to the token stream
/// through [item()][Tokens::item()].
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
/// use genco::tokens::{Item, ItemStr};
///
/// # fn main() -> genco::fmt::Result {
/// let foo = Item::Literal(ItemStr::Static("foo"));
/// let bar = Item::Literal(ItemStr::Box("bar".into()));
///
/// let result: Tokens = quote!(#foo #bar baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
///
/// assert_eq!{
///     vec![
///         Item::Literal(ItemStr::Static("foo")),
///         Item::Space,
///         Item::Literal(ItemStr::Box("bar".into())),
///         Item::Space,
///         Item::Literal(ItemStr::Static("baz")),
///     ] as Vec<Item<()>>,
///     result,
/// };
/// # Ok(())
/// # }
/// ```
impl<L> FormatInto<L> for Item<L>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(self);
    }
}

impl<L> std::fmt::Debug for Item<L>
where
    L: Lang,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(s) => write!(fmt, "Literal({:?})", s),
            Self::Lang(item) => write!(fmt, "Lang({:?})", item),
            Self::Register(item) => write!(fmt, "Register({:?})", item),
            Self::Push => write!(fmt, "Push"),
            Self::Line => write!(fmt, "Line"),
            Self::Space => write!(fmt, "Space"),
            Self::Indentation(n) => write!(fmt, "Indentation({:?})", n),
            Self::OpenQuote(has_eval) => write!(fmt, "OpenQuote({:?})", has_eval),
            Self::CloseQuote => write!(fmt, "CloseQuote"),
            Self::OpenEval => write!(fmt, "OpenEval"),
            Self::CloseEval => write!(fmt, "CloseEval"),
        }
    }
}

impl<L> Clone for Item<L>
where
    L: Lang,
{
    fn clone(&self) -> Self {
        match self {
            Self::Literal(literal) => Self::Literal(literal.clone()),
            Self::Lang(lang) => Self::Lang(lang.__lang_item_clone()),
            Self::Register(lang) => Self::Register(lang.__lang_item_clone()),
            Self::Push => Self::Push,
            Self::Line => Self::Line,
            Self::Space => Self::Space,
            Self::Indentation(n) => Self::Indentation(*n),
            Self::OpenQuote(has_eval) => Self::OpenQuote(*has_eval),
            Self::CloseQuote => Self::CloseQuote,
            Self::OpenEval => Self::OpenEval,
            Self::CloseEval => Self::CloseEval,
        }
    }
}

impl<L> cmp::PartialEq for Item<L>
where
    L: Lang,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(a), Self::Literal(b)) => a == b,
            (Self::Lang(a), Self::Lang(b)) => a.__lang_item_eq(&**b),
            (Self::Register(a), Self::Register(b)) => a.__lang_item_eq(&**b),
            (Self::Push, Self::Push) => true,
            (Self::Line, Self::Line) => true,
            (Self::Space, Self::Space) => true,
            (Self::Indentation(a), Self::Indentation(b)) => *a == *b,
            (Self::OpenQuote(a), Self::OpenQuote(b)) => *a == *b,
            (Self::CloseQuote, Self::CloseQuote) => true,
            (Self::OpenEval, Self::OpenEval) => true,
            (Self::CloseEval, Self::CloseEval) => true,
            _ => false,
        }
    }
}

impl<L> cmp::Eq for Item<L> where L: Lang {}
