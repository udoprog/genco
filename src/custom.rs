//! Trait used for custom element.

use std::fmt;
use super::formatter::Formatter;

pub trait Custom {
    /// Extra data associated with building a formatting element.
    type Extra;

    fn format(&self, _out: &mut Formatter, _extra: &mut Self::Extra, _level: usize) -> fmt::Result {
        Ok(())
    }

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_str(input)
    }
}

/// Dummy implementation for unit.
impl Custom for () {
    type Extra = ();
}
