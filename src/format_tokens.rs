//! Converter traits for things that can be converted into tokens.

use super::{Cons, Element, ErasedElement, Lang, Tokens};

/// Helper trait to convert something into tokens.
pub trait FormatTokens<'el, C> {
    /// Convert the type into tokens.
    fn into_tokens(self, tokens: &mut Tokens<'el, C>);

    /// Hint to test if we are empty.
    fn is_empty(&self) -> bool {
        false
    }
}

impl<'el, C> FormatTokens<'el, C> for Tokens<'el, C> {
    fn into_tokens(self, tokens: &mut Self) {
        tokens.elements.extend(self.elements);
    }

    fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

/// Convert collection to tokens.
impl<'el, C> FormatTokens<'el, C> for Vec<Tokens<'el, C>> {
    fn into_tokens(self, tokens: &mut Tokens<'el, C>) {
        for t in self {
            tokens.elements.extend(t.elements);
        }
    }

    fn is_empty(&self) -> bool {
        self.iter().all(|t| t.is_empty())
    }
}

/// Convert element to tokens.
impl<'el, C> FormatTokens<'el, C> for Element<'el, C> {
    fn into_tokens(self, tokens: &mut Tokens<'el, C>) {
        tokens.elements.push(self);
    }
}

/// Convert an erased element to tokens.
impl<'el, C> FormatTokens<'el, C> for ErasedElement<'el> {
    fn into_tokens(self, tokens: &mut Tokens<'el, C>) {
        tokens.elements.push(self.into());
    }
}

/// Convert custom elements.
impl<'el, C> FormatTokens<'el, C> for C
where
    C: Lang<'el>,
{
    fn into_tokens(self, tokens: &mut Tokens<'el, C>) {
        tokens.elements.push(self.into())
    }
}

/// Convert custom elements.
impl<'el, C> FormatTokens<'el, C> for &'el C
where
    C: Lang<'el>,
{
    fn into_tokens(self, tokens: &mut Tokens<'el, C>) {
        tokens.elements.push(self.into())
    }
}

/// Convert borrowed strings.
impl<'el, C> FormatTokens<'el, C> for &'el str {
    fn into_tokens(self, tokens: &mut Tokens<'el, C>) {
        tokens.elements.push(self.into());
    }
}

/// Convert borrowed strings.
impl<'el, C> FormatTokens<'el, C> for &'el String {
    fn into_tokens(self, tokens: &mut Tokens<'el, C>) {
        tokens.elements.push(self.as_str().into());
    }
}

/// Convert strings.
impl<'el, C> FormatTokens<'el, C> for String {
    fn into_tokens(self, tokens: &mut Tokens<'el, C>) {
        tokens.elements.push(self.into());
    }
}

/// Convert stringy things.
impl<'el, C> FormatTokens<'el, C> for Cons<'el> {
    fn into_tokens(self, tokens: &mut Tokens<'el, C>) {
        tokens.elements.push(self.into());
    }
}

/// Convert stringy things.
impl<'el, C, T> FormatTokens<'el, C> for Option<T>
where
    T: FormatTokens<'el, C>,
{
    fn into_tokens(self, tokens: &mut Tokens<'el, C>) {
        if let Some(inner) = self {
            inner.into_tokens(tokens);
        }
    }
}
