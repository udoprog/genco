//! Helper trait to treat different containers as immediate targets for tokens.

use crate::{Config, Formatter, Lang, Tokens};
use std::fmt;

/// Helper trait to write tokens immediately to containers.
pub trait WriteTokens {
    /// Write the given tokens to the container.
    fn write_tokens<'el, L: Lang<'el>>(
        &mut self,
        tokens: Tokens<'el, L>,
        config: &mut L::Config,
    ) -> fmt::Result;

    /// Write the given tokens to the container as a file.
    fn write_file<'el, L: Lang<'el>>(
        &mut self,
        tokens: Tokens<'el, L>,
        config: &mut L::Config,
    ) -> fmt::Result;
}

impl<W: fmt::Write> WriteTokens for W {
    /// Write token with the given configuration.
    fn write_tokens<'el, L: Lang<'el>>(
        &mut self,
        tokens: Tokens<'el, L>,
        config: &mut L::Config,
    ) -> fmt::Result {
        let mut formatter = Formatter::new(self);
        formatter.indentation = config.indentation();
        tokens.format(&mut formatter, config, 0usize)
    }

    /// Write a a file with the given configuration.
    fn write_file<'el, L: Lang<'el>>(
        &mut self,
        tokens: Tokens<'el, L>,
        config: &mut L::Config,
    ) -> fmt::Result {
        let mut formatter = Formatter::new(self);
        formatter.indentation = config.indentation();
        L::write_file(tokens, &mut formatter, config, 0usize)?;
        formatter.new_line_unless_empty()?;
        Ok(())
    }
}
