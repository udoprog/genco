//! A single element

use crate::lang::LangBox;
use crate::tokens;
use crate::Tokens;
use std::cmp;
use std::num::NonZeroI16;
use std::rc::Rc;

/// A single element in a set of tokens.
pub enum Item {
    /// A literal item.
    /// Is added as a raw string to the stream of tokens.
    Literal(tokens::ItemStr),
    /// A language-specific boxed item.
    LangBox(LangBox),
    /// A language-specific boxed item that is not rendered.
    Registered(LangBox),
    /// Push a new line unless the current line is empty.
    Push,
    /// Unconditionally push a line.
    Line,
    /// Space between language items. Typically a single space.
    ///
    /// Multiple spacings in sequence are collapsed into one.
    /// A spacing does nothing if at the beginning of a line.
    Space,
    /// Manage indentation.
    Indentation(NonZeroI16),
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

impl tokens::FormatInto for Item {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.item(self);
    }
}

impl std::fmt::Debug for Item {
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

impl From<String> for Item {
    fn from(value: String) -> Self {
        Item::Literal(value.into())
    }
}

impl<'a> From<&'a str> for Item {
    fn from(value: &'a str) -> Self {
        Item::Literal(value.into())
    }
}

impl From<Rc<String>> for Item {
    fn from(value: Rc<String>) -> Self {
        Item::Literal(value.into())
    }
}

impl From<tokens::ItemStr> for Item {
    fn from(value: tokens::ItemStr) -> Self {
        Item::Literal(value)
    }
}

impl<'a> From<&'a Item> for Item {
    fn from(value: &'a Item) -> Self {
        value.clone()
    }
}

impl From<Rc<Item>> for Item {
    fn from(value: Rc<Item>) -> Self {
        (*value).clone()
    }
}

impl Clone for Item {
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

impl cmp::PartialEq for Item {
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

impl cmp::Eq for Item {}

#[cfg(test)]
mod tests {
    use super::Item;

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<Item<()>>(), 32);
    }
}
