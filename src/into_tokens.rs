//! Converter traits for things that can be converted into tokens.

use super::tokens::Tokens;

/// Helper trait to convert something into tokens.
pub trait IntoTokens<'el, C> {
    /// Convert the type into tokens.
    fn into_tokens(self) -> Tokens<'el, C>;
}
