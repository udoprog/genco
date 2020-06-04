//! Converter traits for things that can be converted into tokens.

use super::{Cons, Element, ErasedElement, Lang, Tokens};
use std::rc::Rc;

/// Helper trait to convert something into tokens.
pub trait FormatTokens<'el, L>
where
    L: Lang,
{
    /// Convert the type into tokens.
    fn format_tokens(self, tokens: &mut Tokens<'el, L>);

    /// Convert into tokens.
    fn into_tokens(self) -> Tokens<'el, L>
    where
        Self: Sized,
    {
        let mut tokens = Tokens::new();
        self.format_tokens(&mut tokens);
        tokens
    }

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

impl<'el, L> FormatTokens<'el, L> for &'el Tokens<'el, L>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        tokens.elements.extend(self.elements.iter().cloned());
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

/// Convert refcounted strings.
impl<'el, L> FormatTokens<'el, L> for Rc<String>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        tokens.elements.push(self.into());
    }
}

/// Convert reference to refcounted strings.
impl<'el, L> FormatTokens<'el, L> for &'el Rc<String>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        tokens.elements.push(Cons::Borrowed(self.as_str()).into());
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

macro_rules! impl_display {
    ($($ty:ty),*) => {
        $(
            impl<'el, L> FormatTokens<'el, L> for $ty
            where
                L: Lang,
            {
                fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
                    tokens.append(self.to_string());
                }
            }
        )*
    };
}

impl_display!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
