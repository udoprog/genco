//! Converter traits for things that can be converted into tokens.

use super::{Cons, Element, ErasedElement, Lang, Tokens};

/// Helper trait to convert something into tokens.
pub trait FormatTokens<'el, L>
where
    L: Lang,
{
    /// Convert the type into tokens.
    fn format_tokens(self, tokens: &mut Tokens<'el, L>);

    /// Hint to test if we are empty.
    fn is_empty(&self) -> bool {
        false
    }
}

impl<'el, L> FormatTokens<'el, L> for Tokens<'el, L>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Self) {
        tokens.elements.extend(self.elements);
    }

    fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

/// Convert collection to tokens.
impl<'el, L> FormatTokens<'el, L> for Vec<Tokens<'el, L>>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        for t in self {
            tokens.elements.extend(t.elements);
        }
    }

    fn is_empty(&self) -> bool {
        self.iter().all(|t| t.is_empty())
    }
}

/// Convert element to tokens.
impl<'el, L> FormatTokens<'el, L> for Element<'el, L>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        tokens.elements.push(self);
    }
}

/// Convert an erased element to tokens.
impl<'el, L> FormatTokens<'el, L> for ErasedElement<'el>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        tokens.elements.push(self.into());
    }
}

/// Convert borrowed strings.
impl<'el, L> FormatTokens<'el, L> for &'el str
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        tokens.elements.push(self.into());
    }
}

/// Convert borrowed strings.
impl<'el, L> FormatTokens<'el, L> for &'el String
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        tokens.elements.push(self.as_str().into());
    }
}

/// Convert strings.
impl<'el, L> FormatTokens<'el, L> for String
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        tokens.elements.push(self.into());
    }
}

/// Convert stringy things.
impl<'el, L> FormatTokens<'el, L> for Cons<'el>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        tokens.elements.push(self.into());
    }
}

/// Convert stringy things.
impl<'el, L, T> FormatTokens<'el, L> for Option<T>
where
    L: Lang,
    T: FormatTokens<'el, L>,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        if let Some(inner) = self {
            inner.format_tokens(tokens);
        }
    }
}
