//! Trait used for custom element.

use crate::{Config, Formatter, Tokens};
use std::fmt;

/// Trait that must be implemented by custom elements.
pub trait Custom<'el>
where
    Self: Sized,
{
    /// Configuration associated with building a formatting element.
    type Config: Config;

    /// Format the custom element.
    fn format(
        &self,
        _out: &mut Formatter,
        _config: &mut Self::Config,
        _level: usize,
    ) -> fmt::Result {
        Ok(())
    }

    /// Performing quoting according to convention set by custom element.
    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_str(input)
    }

    /// Write a file according to convention by custom element.
    fn write_file(
        tokens: Tokens<'el, Self>,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        tokens.format(out, config, level)
    }
}

/// Dummy implementation for unit.
impl<'el> Custom<'el> for () {
    type Config = ();
}
