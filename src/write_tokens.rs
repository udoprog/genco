//! Helper trait to treat different containers as immediate targets for tokens.

use std::fmt;
use super::tokens::Tokens;
use super::formatter::WriteFormatter;
use super::custom::Custom;

/// Helper trait to write tokens immediately to containers.
pub trait WriteTokens {
    /// Write the given tokens to the container.
    fn write_tokens<'element, C: Custom>(
        &mut self,
        tokens: Tokens<'element, C>,
        extra: &mut C::Extra,
    ) -> fmt::Result;

    /// Write the given tokens to the container as a file.
    fn write_file<'element, C: Custom>(
        &mut self,
        tokens: Tokens<'element, C>,
        extra: &mut C::Extra,
    ) -> fmt::Result;
}

impl WriteTokens for String {
    fn write_tokens<'element, C: Custom>(
        &mut self,
        tokens: Tokens<'element, C>,
        extra: &mut C::Extra,
    ) -> fmt::Result {
        tokens.format(&mut WriteFormatter::new(self), extra, 0usize)
    }

    fn write_file<'element, C: Custom>(
        &mut self,
        tokens: Tokens<'element, C>,
        extra: &mut C::Extra,
    ) -> fmt::Result {
        C::write_file(tokens, &mut WriteFormatter::new(self), extra, 0usize)
    }
}
