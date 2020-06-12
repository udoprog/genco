use super::{Item, Lang, Tokens};
use std::rc::Rc;

/// Trait for formatting something as a stream of tokens.
///
/// Things implementing [FormatTokens] can be used as arguments for
/// [interpolation] in the [quote!] macro.
///
/// [quote!]: macro.quote.html
/// [interpolation]: macro.quote.html#interpolation
pub trait FormatTokens<L>
where
    L: Lang,
{
    /// Convert the type into tokens in-place.
    ///
    /// # Examples
    fn format_tokens(self, tokens: &mut Tokens<L>);
}

/// Construct a formatter from a function.
pub fn from_fn<F, L>(f: F) -> FormatFn<F>
where
    F: FnOnce(&mut Tokens<L>),
    L: Lang,
{
    FormatFn { f }
}

/// A captured function used for formatting.
pub struct FormatFn<F> {
    f: F,
}

impl<L, F> FormatTokens<L> for FormatFn<F>
where
    L: Lang,
    F: FnOnce(&mut Tokens<L>),
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        (self.f)(tokens);
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
        tokens.item(self);
    }
}

/// Convert borrowed strings.
impl<'a, L> FormatTokens<L> for &'a str
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.item(self.to_string().into());
    }
}

/// Convert borrowed strings.
impl<'a, L> FormatTokens<L> for &'a String
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.item(self.clone().into());
    }
}

/// Convert strings.
impl<L> FormatTokens<L> for String
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.item(self.into());
    }
}

/// Convert refcounted strings.
impl<L> FormatTokens<L> for Rc<String>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.item(self.into());
    }
}

/// Convert reference to refcounted strings.
impl<'a, L> FormatTokens<L> for &'a Rc<String>
where
    L: Lang,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.item(self.clone().into());
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
