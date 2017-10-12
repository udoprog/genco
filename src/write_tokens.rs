//! Helper trait to treat different containers as immediate targets for tokens.

use std::fmt;
use super::tokens::Tokens;
use super::formatter::Formatter;
use super::custom::Custom;

/// Helper trait to write tokens immediately to containers.
pub trait WriteTokens {
    /// Write the given tokens to the container.
    fn write_tokens<'el, C: Custom>(
        &mut self,
        tokens: Tokens<'el, C>,
        extra: &mut C::Extra,
    ) -> fmt::Result;

    /// Write the given tokens to the container as a file.
    fn write_file<'el, C: Custom>(
        &mut self,
        tokens: Tokens<'el, C>,
        extra: &mut C::Extra,
    ) -> fmt::Result;
}

impl<W: fmt::Write> WriteTokens for W {
    fn write_tokens<'el, C: Custom>(
        &mut self,
        tokens: Tokens<'el, C>,
        extra: &mut C::Extra,
    ) -> fmt::Result {
        tokens.format(&mut Formatter::new(self), extra, 0usize)
    }

    fn write_file<'el, C: Custom>(
        &mut self,
        tokens: Tokens<'el, C>,
        extra: &mut C::Extra,
    ) -> fmt::Result {
        let mut formatter = Formatter::new(self);
        C::write_file(tokens, &mut formatter, extra, 0usize)?;
        formatter.new_line_unless_empty()?;
        Ok(())
    }
}
