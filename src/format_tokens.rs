//! Converter traits for things that can be converted into tokens.

use super::{Item, Lang, Tokens};
use std::rc::Rc;

/// Helper trait to convert something into tokens.
pub trait FormatTokens<L>
where
    L: Lang,
{
    /// Convert the type into tokens.
    fn format_tokens(self, tokens: &mut Tokens<L>);

    /// Convert into tokens.
    fn into_tokens(self) -> Tokens<L>
    where
        Self: Sized,
    {
        let mut tokens = Tokens::new();
        self.format_tokens(&mut tokens);
        tokens
    }
}

impl<L> FormatTokens<L> for Tokens<L>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Self) {
        tokens.extend(self);
    }
}

impl<'a, L> FormatTokens<L> for &'a Tokens<L>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.extend(self.iter().cloned());
    }
}

/// Convert collection to tokens.
impl<L> FormatTokens<L> for Vec<Tokens<L>>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        for t in self {
            tokens.extend(t);
        }
    }
}

/// Convert element to tokens.
impl<L> FormatTokens<L> for Item<L>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.push_item(self);
    }
}

/// Convert borrowed strings.
impl<'a, L> FormatTokens<L> for &'a str
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.push_item(self.to_string().into());
    }
}

/// Convert borrowed strings.
impl<'a, L> FormatTokens<L> for &'a String
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.push_item(self.clone().into());
    }
}

/// Convert strings.
impl<L> FormatTokens<L> for String
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.push_item(self.into());
    }
}

/// Convert refcounted strings.
impl<L> FormatTokens<L> for Rc<String>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.push_item(self.into());
    }
}

/// Convert reference to refcounted strings.
impl<'a, L> FormatTokens<L> for &'a Rc<String>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.push_item(self.clone().into());
    }
}

/// Convert stringy things.
impl<L, T> FormatTokens<L> for Option<T>
where
    L: Lang,
    T: FormatTokens<L>,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        if let Some(inner) = self {
            inner.format_tokens(tokens);
        }
    }
}

/// Unit implementation of format tokens. Does nothing.
impl<L> FormatTokens<L> for ()
where
    L: Lang,
{
    #[inline]
    fn format_tokens(self, _: &mut Tokens<L>) {}
}

macro_rules! impl_display {
    ($($ty:ty),*) => {
        $(
            impl<L> FormatTokens<L> for $ty
            where
                L: Lang,
            {
                fn format_tokens(self, tokens: &mut Tokens<L>) {
                    tokens.append(self.to_string());
                }
            }
        )*
    };
}

impl_display!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize, usize);
