//! Trait used for custom element.

use std::fmt;
use super::formatter::Formatter;
use super::tokens::Tokens;

pub trait Custom
where
    Self: Sized,
{
    /// Extra data associated with building a formatting element.
    type Extra: Default;

    fn format(&self, _out: &mut Formatter, _extra: &mut Self::Extra, _level: usize) -> fmt::Result {
        Ok(())
    }

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_str(input)
    }

    fn write_file<'element>(
        tokens: Tokens<'element, Self>,
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
