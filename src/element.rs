//! A single element

use super::formatter::Formatter;
use super::custom::Custom;
use std::fmt;
use super::tokens::Tokens;
use super::contained::Contained;

/// A single element in a set of tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element<'element, C: 'element> {
    /// Append the given set of tokens.
    Append(Contained<'element, Tokens<'element, C>>),
    /// Push the owned set of tokens, adding a newline if current line is not empty.
    Push(Contained<'element, Tokens<'element, C>>),
    /// Nested on indentation level.
    Nested(Contained<'element, Tokens<'element, C>>),
    /// Single-space spacing.
    Spacing,
    /// New line if needed.
    LineSpacing,
    /// A borrowed string.
    BorrowedLiteral(&'element str),
    /// An owned string.
    OwnedLiteral(String),
    /// A borrowed quoted string.
    BorrowedQuoted(&'element str),
    /// An owned quoted string.
    OwnedQuoted(String),
    /// Language-specific items.
    Custom(C),
}

impl<'element, C: Custom> Element<'element, C> {
    /// Format the given element.
    pub fn format(&self, out: &mut Formatter, extra: &mut C::Extra, level: usize) -> fmt::Result {
        use self::Element::*;

        match *self {
            Append(ref tokens) => {
                tokens.as_ref().format(out, extra, level)?;
            }
            Push(ref tokens) => {
                out.new_line_unless_empty()?;
                tokens.as_ref().format(out, extra, level)?;
            }
            Nested(ref tokens) => {
                out.new_line_unless_empty()?;

                out.indent();
                tokens.as_ref().format(out, extra, level + 1usize)?;
                out.unindent();
            }
            LineSpacing => {
                out.new_line_unless_empty()?;
                out.new_line()?;
            }
            Spacing => {
                out.write_str(" ")?;
            }
            BorrowedLiteral(ref literal) => {
                out.write_str(literal)?;
            }
            OwnedLiteral(ref literal) => {
                out.write_str(literal.as_str())?;
            }
            BorrowedQuoted(ref literal) => {
                C::quote_string(out, literal)?;
            }
            OwnedQuoted(ref literal) => {
                C::quote_string(out, literal.as_str())?;
            }
            Custom(ref custom) => {
                custom.format(out, extra, level)?;
            }
        }

        Ok(())
    }
}

impl<'element, C> From<&'element str> for Element<'element, C> {
    fn from(value: &'element str) -> Self {
        Element::BorrowedLiteral(value)
    }
}

impl<'element, C> From<String> for Element<'element, C> {
    fn from(value: String) -> Self {
        Element::OwnedLiteral(value)
    }
}

impl<'element, C> From<Tokens<'element, C>> for Element<'element, C> {
    fn from(value: Tokens<'element, C>) -> Self {
        Element::Append(Contained::Owned(value))
    }
}

impl<'element, C: Custom> From<C> for Element<'element, C> {
    fn from(value: C) -> Self {
        Element::Custom(value)
    }
}
