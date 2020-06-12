use crate::lang::Lang;
use crate::Tokens;
use std::rc::Rc;

/// Trait for types that can be formatted in-place into a token stream.
///
/// Things implementing [FormatInto] can be used as arguments for
/// [interpolation] in the [quote!] macro.
///
/// [from_fn()] is a helper function which simplifies the task of creating a
/// [FormatInto] implementation on the fly.
///
/// [from_fn()]: crate::tokens::from_fn()
/// [quote!]: macro.quote.html
/// [interpolation]: macro.quote.html#interpolation
///
/// # Examples
///
/// ```rust
/// # fn main() -> genco::fmt::Result {
/// use genco::quote_in;
/// use genco::tokens::{ItemStr, FormatInto, from_fn, static_literal};
/// use genco::lang::Lang;
///
/// fn comment<L>(s: impl Into<ItemStr>) -> impl FormatInto<L>
/// where
///     L: Lang
/// {
///     from_fn(move |tokens| {
///         let s = s.into();
///         quote_in!(*tokens => #(static_literal("//")) #s);
///     })
/// }
/// # Ok(())
/// # }
/// ```
pub trait FormatInto<L>
where
    L: Lang,
{
    /// Convert the type into tokens in-place.
    ///
    /// # Examples
    fn format_into(self, tokens: &mut Tokens<L>);
}

impl<L> FormatInto<L> for Tokens<L>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Self) {
        tokens.extend(self);
    }
}

impl<'a, L> FormatInto<L> for &'a Tokens<L>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.extend(self.iter().cloned());
    }
}

/// Convert collection to tokens.
impl<L> FormatInto<L> for Vec<Tokens<L>>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        for t in self {
            tokens.extend(t);
        }
    }
}

/// Convert borrowed strings.
impl<'a, L> FormatInto<L> for &'a str
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(self.to_string().into());
    }
}

/// Convert borrowed strings.
impl<'a, L> FormatInto<L> for &'a String
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(self.clone().into());
    }
}

/// Convert strings.
impl<L> FormatInto<L> for String
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(self.into());
    }
}

/// Convert refcounted strings.
impl<L> FormatInto<L> for Rc<String>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(self.into());
    }
}

/// Convert reference to refcounted strings.
impl<'a, L> FormatInto<L> for &'a Rc<String>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(self.clone().into());
    }
}

/// Convert stringy things.
impl<L, T> FormatInto<L> for Option<T>
where
    L: Lang,
    T: FormatInto<L>,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        if let Some(inner) = self {
            inner.format_into(tokens);
        }
    }
}

/// Unit implementation of format tokens. Does nothing.
impl<L> FormatInto<L> for ()
where
    L: Lang,
{
    #[inline]
    fn format_into(self, _: &mut Tokens<L>) {}
}

macro_rules! impl_display {
    ($($ty:ty),*) => {
        $(
            impl<L> FormatInto<L> for $ty
            where
                L: Lang,
            {
                fn format_into(self, tokens: &mut Tokens<L>) {
                    tokens.append(self.to_string());
                }
            }
        )*
    };
}

impl_display!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize, usize);
