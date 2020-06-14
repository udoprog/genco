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
/// fn comment(s: impl Into<ItemStr>) -> impl FormatInto
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
pub trait FormatInto {
    /// Convert the type into tokens in-place.
    ///
    /// # Examples
    fn format_into(self, tokens: &mut Tokens);
}

impl FormatInto for Tokens {
    fn format_into(self, tokens: &mut Self) {
        tokens.extend(self);
    }
}

impl<'a> FormatInto for &'a Tokens {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.extend(self.iter().cloned());
    }
}

/// Convert collection to tokens.
impl FormatInto for Vec<Tokens> {
    fn format_into(self, tokens: &mut Tokens) {
        for t in self {
            tokens.extend(t);
        }
    }
}

/// Convert borrowed strings.
impl<'a> FormatInto for &'a str {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.item(self.to_string().into());
    }
}

/// Convert borrowed strings.
impl<'a> FormatInto for &'a String {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.item(self.clone().into());
    }
}

/// Convert strings.
impl FormatInto for String {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.item(self.into());
    }
}

/// Convert refcounted strings.
impl FormatInto for Rc<String> {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.item(self.into());
    }
}

/// Convert reference to refcounted strings.
impl<'a> FormatInto for &'a Rc<String> {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.item(self.clone().into());
    }
}

/// Convert stringy things.
impl<T> FormatInto for Option<T>
where
    T: FormatInto,
{
    fn format_into(self, tokens: &mut Tokens) {
        if let Some(inner) = self {
            inner.format_into(tokens);
        }
    }
}

/// Unit implementation of format tokens. Does nothing.
impl FormatInto for () {
    #[inline]
    fn format_into(self, _: &mut Tokens) {}
}

macro_rules! impl_display {
    ($($ty:ty),*) => {
        $(
            impl FormatInto for $ty {
                fn format_into(self, tokens: &mut Tokens) {
                    tokens.append(self.to_string());
                }
            }
        )*
    };
}

impl_display!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize, usize);
