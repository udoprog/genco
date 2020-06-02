//! A single element

use crate::{Cons, ErasedElement, Formatter, Lang, LangBox, LangItem as _};
use std::fmt;
use std::rc::Rc;

/// A single element in a set of tokens.
#[derive(Debug, Clone)]
pub enum Element<'el, L>
where
    L: Lang,
{
    /// A refcounted member.
    Rc(Rc<Element<'el, L>>),
    /// A borrowed element.
    Borrowed(&'el Element<'el, L>),
    /// A borrowed string.
    Literal(Cons<'el>),
    /// A borrowed quoted string.
    Quoted(Cons<'el>),
    /// Language-specific boxed items.
    LangBox(LangBox<'el, L>),
    /// A custom element that is not rendered.
    Registered(LangBox<'el, L>),
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

impl<'el, L> Element<'el, L>
where
    L: Lang,
{
    /// Test if the element is none.
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }
}

impl<'el, L> From<ErasedElement<'el>> for Element<'el, L>
where
    L: Lang,
{
    fn from(erased: ErasedElement<'el>) -> Self {
        match erased {
            ErasedElement::Quoted(text) => Self::Quoted(text),
        }
    }
}

impl<'el, L> Element<'el, L>
where
    L: Lang,
{
    /// Format the given element.
    pub fn format(&self, out: &mut Formatter, config: &mut L::Config, level: usize) -> fmt::Result {
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
                L::quote_string(out, literal.as_ref())?;
            }
            LangBox(ref lang) => {
                lang.format(out, config, level)?;
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

impl<'el, L> From<String> for Element<'el, L>
where
    L: Lang,
{
    fn from(value: String) -> Self {
        Element::Literal(value.into())
    }
}

impl<'el, L> From<&'el str> for Element<'el, L>
where
    L: Lang,
{
    fn from(value: &'el str) -> Self {
        Element::Literal(value.into())
    }
}

impl<'el, L> From<Rc<String>> for Element<'el, L>
where
    L: Lang,
{
    fn from(value: Rc<String>) -> Self {
        Element::Literal(value.into())
    }
}

impl<'el, L> From<Cons<'el>> for Element<'el, L>
where
    L: Lang,
{
    fn from(value: Cons<'el>) -> Self {
        Element::Literal(value)
    }
}

impl<'el, L> From<&'el Element<'el, L>> for Element<'el, L>
where
    L: Lang,
{
    fn from(value: &'el Element<'el, L>) -> Self {
        Element::Borrowed(value)
    }
}

impl<'el, L> From<Rc<Element<'el, L>>> for Element<'el, L>
where
    L: Lang,
{
    fn from(value: Rc<Element<'el, L>>) -> Self {
        Element::Rc(value)
    }
}
