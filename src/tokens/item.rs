//! A single element

use crate::lang::{Lang, LangBox};
use crate::tokens;
use crate::Tokens;
use std::cmp;
use std::rc::Rc;

/// A single element in a set of tokens.
pub enum Item<L>
where
    L: Lang,
{
    /// A literal item.
    /// Is added as a raw string to the stream of tokens.
    Literal(tokens::ItemStr),
    /// A language-specific boxed item.
    LangBox(LangBox<L>),
    /// A language-specific boxed item that is not rendered.
    Registered(LangBox<L>),
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

impl<L> tokens::FormatInto<L> for Item<L>
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
            Self::LangBox(item) => write!(fmt, "LangBox({:?})", item),
            Self::Registered(item) => write!(fmt, "Registered({:?})", item),
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

impl<L> From<String> for Item<L>
where
    L: Lang,
{
    fn from(value: String) -> Self {
        Item::Literal(value.into())
    }
}

impl<'a, L> From<&'a str> for Item<L>
where
    L: Lang,
{
    fn from(value: &'a str) -> Self {
        Item::Literal(value.into())
    }
}

impl<L> From<Rc<String>> for Item<L>
where
    L: Lang,
{
    fn from(value: Rc<String>) -> Self {
        Item::Literal(value.into())
    }
}

impl<L> From<tokens::ItemStr> for Item<L>
where
    L: Lang,
{
    fn from(value: tokens::ItemStr) -> Self {
        Item::Literal(value)
    }
}

impl<'a, L> From<&'a Item<L>> for Item<L>
where
    L: Lang,
{
    fn from(value: &'a Item<L>) -> Self {
        value.clone()
    }
}

impl<L> From<Rc<Item<L>>> for Item<L>
where
    L: Lang,
{
    fn from(value: Rc<Item<L>>) -> Self {
        (*value).clone()
    }
}

impl<L> Clone for Item<L>
where
    L: Lang,
{
    fn clone(&self) -> Self {
        match self {
            Self::Literal(literal) => Self::Literal(literal.clone()),
            Self::LangBox(lang) => Self::LangBox(lang.clone()),
            Self::Registered(lang) => Self::Registered(lang.clone()),
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
            (Self::LangBox(a), Self::LangBox(b)) => a.__lang_item_eq(&**b),
            (Self::Registered(a), Self::Registered(b)) => a.__lang_item_eq(&**b),
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

#[cfg(test)]
mod tests {
    use super::Item;

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<Item<()>>(), 32);
    }
}
