//! A single element

use crate::{Formatter, ItemStr, Lang, LangBox, LangItem as _};
use std::fmt;
use std::rc::Rc;

/// A single element in a set of tokens.
#[derive(Debug)]
pub enum Element<L>
where
    L: Lang,
{
    /// A refcounted member.
    Rc(Rc<Element<L>>),
    /// A borrowed string.
    Literal(ItemStr),
    /// A borrowed quoted string.
    Quoted(ItemStr),
    /// Language-specific boxed items.
    LangBox(LangBox<L>),
    /// A custom element that is not rendered.
    Registered(LangBox<L>),
    /// Push a new line, unless the current line is empty.
    PushSpacing,
    /// Unconditionally push a line.
    Line,
    /// Spacing between language items.
    Spacing,
    /// Push a new line, unless the current line is empty, then add another line
    /// after that to create an empty line as spacing.
    LineSpacing,
    /// Indent one step.
    Indent,
    /// Unindent one step.
    Unindent,
}

impl<L> Element<L>
where
    L: Lang,
{
    /// Format the given element.
    pub fn format(&self, out: &mut Formatter, config: &mut L::Config, level: usize) -> fmt::Result {
        use self::Element::*;

        match *self {
            Registered(_) => {}
            Rc(ref element) => {
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

impl<L> From<String> for Element<L>
where
    L: Lang,
{
    fn from(value: String) -> Self {
        Element::Literal(value.into())
    }
}

impl<'a, L> From<&'a str> for Element<L>
where
    L: Lang,
{
    fn from(value: &'a str) -> Self {
        Element::Literal(value.into())
    }
}

impl<L> From<Rc<String>> for Element<L>
where
    L: Lang,
{
    fn from(value: Rc<String>) -> Self {
        Element::Literal(value.into())
    }
}

impl<L> From<ItemStr> for Element<L>
where
    L: Lang,
{
    fn from(value: ItemStr) -> Self {
        Element::Literal(value)
    }
}

impl<'a, L> From<&'a Element<L>> for Element<L>
where
    L: Lang,
{
    fn from(value: &'a Element<L>) -> Self {
        value.clone()
    }
}

impl<L> From<Rc<Element<L>>> for Element<L>
where
    L: Lang,
{
    fn from(value: Rc<Element<L>>) -> Self {
        Element::Rc(value)
    }
}

impl<L> Clone for Element<L>
where
    L: Lang,
{
    fn clone(&self) -> Self {
        match self {
            Self::Rc(element) => Self::Rc(element.clone()),
            Self::Literal(literal) => Self::Literal(literal.clone()),
            Self::Quoted(quoted) => Self::Quoted(quoted.clone()),
            Self::LangBox(lang) => Self::LangBox(lang.clone()),
            Self::Registered(lang) => Self::Registered(lang.clone()),
            Self::PushSpacing => Self::PushSpacing,
            Self::Line => Self::Line,
            Self::Spacing => Self::Spacing,
            Self::LineSpacing => Self::LineSpacing,
            Self::Indent => Self::Indent,
            Self::Unindent => Self::Unindent,
        }
    }
}
