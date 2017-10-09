//! A single element

use super::formatter::Formatter;
use super::custom::Custom;
use std::fmt;
use super::tokens::Tokens;
use super::con::Con;
use super::cons::Cons;

use std::rc::Rc;

/// A single element in a set of tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element<'element, C: 'element> {
    /// A refcounted member.
    Rc(Rc<Element<'element, C>>),
    /// A borrowed element.
    Borrowed(&'element Element<'element, C>),
    /// Append the given set of tokens.
    Append(Con<'element, Tokens<'element, C>>),
    /// Push the owned set of tokens, adding a newline if current line is not empty.
    Push(Con<'element, Tokens<'element, C>>),
    /// Nested on indentation level.
    Nested(Con<'element, Tokens<'element, C>>),
    /// Single-space spacing.
    Spacing,
    /// New line if needed.
    LineSpacing,
    /// A borrowed string.
    Literal(Cons<'element>),
    /// A borrowed quoted string.
    Quoted(Cons<'element>),
    /// Language-specific items.
    Custom(Con<'element, C>),
}

impl<'element, C: Custom> Element<'element, C> {
    /// Format the given element.
    pub fn format(&self, out: &mut Formatter, extra: &mut C::Extra, level: usize) -> fmt::Result {
        use self::Element::*;

        match *self {
            Rc(ref element) => {
                element.format(out, extra, level)?;
            }
            Borrowed(element) => {
                element.format(out, extra, level)?;
            }
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
            Literal(ref literal) => {
                out.write_str(literal.as_ref())?;
            }
            Quoted(ref literal) => {
                C::quote_string(out, literal.as_ref())?;
            }
            Custom(ref custom) => {
                custom.as_ref().format(out, extra, level)?;
            }
        }

        Ok(())
    }
}

impl<'element, C: Custom> From<C> for Element<'element, C> {
    fn from(value: C) -> Self {
        Element::Custom(Con::Owned(value))
    }
}

impl<'element, C: Custom> From<&'element C> for Element<'element, C> {
    fn from(value: &'element C) -> Self {
        Element::Custom(Con::Borrowed(value))
    }
}

impl<'element, C> From<String> for Element<'element, C> {
    fn from(value: String) -> Self {
        Element::Literal(Cons::Owned(value))
    }
}

impl<'element, C> From<&'element str> for Element<'element, C> {
    fn from(value: &'element str) -> Self {
        Element::Literal(Cons::Borrowed(value))
    }
}

impl<'element, C> From<Rc<String>> for Element<'element, C> {
    fn from(value: Rc<String>) -> Self {
        Element::Literal(Cons::Rc(value))
    }
}

impl<'element, C> From<&'element Element<'element, C>> for Element<'element, C> {
    fn from(value: &'element Element<'element, C>) -> Self {
        Element::Borrowed(value)
    }
}

impl<'element, C> From<Rc<Element<'element, C>>> for Element<'element, C> {
    fn from(value: Rc<Element<'element, C>>) -> Self {
        Element::Rc(value)
    }
}

impl<'element, C> From<Tokens<'element, C>> for Element<'element, C> {
    fn from(value: Tokens<'element, C>) -> Self {
        Element::Append(Con::Owned(value))
    }
}

impl<'element, C> From<&'element Tokens<'element, C>> for Element<'element, C> {
    fn from(value: &'element Tokens<'element, C>) -> Self {
        Element::Append(Con::Borrowed(value))
    }
}

impl<'element, C> From<Rc<Tokens<'element, C>>> for Element<'element, C> {
    fn from(value: Rc<Tokens<'element, C>>) -> Self {
        Element::Append(Con::Rc(value))
    }
}
