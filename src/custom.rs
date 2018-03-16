//! Trait used for custom element.

use std::fmt;
use super::formatter::Formatter;
use super::tokens::Tokens;

/// Trait that must be implemented by custom elements.
pub trait Custom
where
    Self: Sized,
{
    /// Extra data associated with building a formatting element.
    type Extra;

    /// Format the custom element.
    fn format(&self, _out: &mut Formatter, _extra: &mut Self::Extra, _level: usize) -> fmt::Result {
        Ok(())
    }

    /// Performing quoting according to convention set by custom element.
    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_str(input)
    }

    /// Write a file according to convention by custom element.
    fn write_file<'el>(
        tokens: Tokens<'el, Self>,
        out: &mut Formatter,
        extra: &mut Self::Extra,
        level: usize,
    ) -> fmt::Result {
        tokens.format(out, extra, level)
    }
}

/// Dummy implementation for unit.
impl Custom for () {
    type Extra = ();
}
