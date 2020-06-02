//! A single element

use super::con_::Con;
use crate::{Cons, ErasedElement, Formatter, Lang};
use std::fmt;

use std::rc::Rc;

/// A single element in a set of tokens.
#[derive(Debug, Clone)]
pub enum Element<'el, C> {
    /// A refcounted member.
    Rc(Rc<Element<'el, C>>),
    /// A borrowed element.
    Borrowed(&'el Element<'el, C>),
    /// A borrowed string.
    Literal(Cons<'el>),
    /// A borrowed quoted string.
    Quoted(Cons<'el>),
    /// Language-specific items.
    Lang(Con<'el, C>),
    /// A custom element that is not rendered.
    Registered(Con<'el, C>),
    /// Push an empty line.
    PushSpacing,
    /// Unconditionally push a line.
    Line,
    /// Single-space spacing.
    Spacing,
    /// New line if needed.
    LineSpacing,
    /// Indent.
    Indent,
    /// Unindent.
    Unindent,
    /// Empty element which renders nothing.
    None,
}

impl<'el, C> Element<'el, C> {
    /// Test if the element is none.
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }
}

impl<'el, C: 'el> From<ErasedElement<'el>> for Element<'el, C> {
    fn from(erased: ErasedElement<'el>) -> Self {
        match erased {
            ErasedElement::Quoted(text) => Self::Quoted(text),
        }
    }
}

impl<'el, C: Lang<'el>> Element<'el, C> {
    /// Format the given element.
    pub fn format(&self, out: &mut Formatter, config: &mut C::Config, level: usize) -> fmt::Result {
        use self::Element::*;

        match *self {
            Registered(_) => {}
            None => {}
            Rc(ref element) => {
                element.format(out, config, level)?;
            }
            Borrowed(element) => {
                element.format(out, config, level)?;
            }
            Literal(ref literal) => {
                out.write_str(literal.as_ref())?;
            }
            Quoted(ref literal) => {
                C::quote_string(out, literal.as_ref())?;
            }
            Lang(ref custom) => {
                custom.as_ref().format(out, config, level)?;
            }
            // whitespace below
            PushSpacing => {
                out.new_line_unless_empty()?;
            }
            Line => {
                out.new_line()?;
            }
            LineSpacing => {
                out.new_line_unless_empty()?;
                out.new_line()?;
            }
            Spacing => {
                out.write_str(" ")?;
            }
            Indent => {
                out.indent();
                out.new_line_unless_empty()?;
            }
            Unindent => {
                out.unindent();
                out.new_line_unless_empty()?;
            }
        }

        Ok(())
    }
}

impl<'el, C: Lang<'el>> From<C> for Element<'el, C> {
    fn from(value: C) -> Self {
        Element::Lang(Con::Owned(value))
    }
}

impl<'el, C: Lang<'el>> From<&'el C> for Element<'el, C> {
    fn from(value: &'el C) -> Self {
        Element::Lang(Con::Borrowed(value))
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
