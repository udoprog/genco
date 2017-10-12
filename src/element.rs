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
pub enum Element<'el, C: 'el> {
    /// A refcounted member.
    Rc(Rc<Element<'el, C>>),
    /// A borrowed element.
    Borrowed(&'el Element<'el, C>),
    /// Append the given set of tokens.
    Append(Con<'el, Tokens<'el, C>>),
    /// Push an empty line.
    PushLine,
    /// Push the owned set of tokens, adding a newline if current line is not empty.
    Push(Con<'el, Tokens<'el, C>>),
    /// Nested on indentation level.
    Nested(Con<'el, Tokens<'el, C>>),
    /// Single-space spacing.
    Spacing,
    /// New line if needed.
    LineSpacing,
    /// A borrowed string.
    Literal(Cons<'el>),
    /// A borrowed quoted string.
    Quoted(Cons<'el>),
    /// Language-specific items.
    Custom(Con<'el, C>),
}

impl<'el, C: Custom> Element<'el, C> {
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
            PushLine => {
                out.new_line_unless_empty()?;
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

impl<'el, C: Custom> From<C> for Element<'el, C> {
    fn from(value: C) -> Self {
        Element::Custom(Con::Owned(value))
    }
}

impl<'el, C: Custom> From<&'el C> for Element<'el, C> {
    fn from(value: &'el C) -> Self {
        Element::Custom(Con::Borrowed(value))
    }
}

impl<'el, C> From<String> for Element<'el, C> {
    fn from(value: String) -> Self {
        Element::Literal(value.into())
    }
}

impl<'el, C> From<&'el str> for Element<'el, C> {
    fn from(value: &'el str) -> Self {
        Element::Literal(value.into())
    }
}

impl<'el, C> From<Rc<String>> for Element<'el, C> {
    fn from(value: Rc<String>) -> Self {
        Element::Literal(value.into())
    }
}

impl<'el, C> From<Cons<'el>> for Element<'el, C> {
    fn from(value: Cons<'el>) -> Self {
        Element::Literal(value)
    }
}

impl<'el, C> From<&'el Element<'el, C>> for Element<'el, C> {
    fn from(value: &'el Element<'el, C>) -> Self {
        Element::Borrowed(value)
    }
}

impl<'el, C> From<Rc<Element<'el, C>>> for Element<'el, C> {
    fn from(value: Rc<Element<'el, C>>) -> Self {
        Element::Rc(value)
    }
}

impl<'el, C> From<Tokens<'el, C>> for Element<'el, C> {
    fn from(value: Tokens<'el, C>) -> Self {
        Element::Append(Con::Owned(value))
    }
}

impl<'el, C> From<&'el Tokens<'el, C>> for Element<'el, C> {
    fn from(value: &'el Tokens<'el, C>) -> Self {
        Element::Append(Con::Borrowed(value))
    }
}

impl<'el, C> From<Rc<Tokens<'el, C>>> for Element<'el, C> {
    fn from(value: Rc<Tokens<'el, C>>) -> Self {
        Element::Append(Con::Rc(value))
    }
}
