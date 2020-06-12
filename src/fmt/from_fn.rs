use crate::{FormatTokens, Lang, Tokens};

/// Construct a formatter from a function.
/// 
/// # Examples
/// 
/// ```rust
/// # fn main() -> genco::fmt::Result {
/// use genco::{ItemStr, fmt, FormatTokens, quote_in, Lang};
///
/// fn comment<L>(s: impl Into<ItemStr>) -> impl FormatTokens<L>
/// where
///     L: Lang
/// {
///     fmt::from_fn(move |tokens| {
///         let s = s.into();
///         quote_in!(*tokens => #(ItemStr::Static("//")) #s);
///     })
/// }
/// # Ok(())
/// # }
/// ```
pub fn from_fn<F, L>(f: F) -> FromFn<F>
where
    F: FnOnce(&mut Tokens<L>),
    L: Lang,
{
    FromFn { f }
}

/// A captured function used for formatting tokens.
/// 
/// Created using [from_fn()].
pub struct FromFn<F> {
    f: F,
}

impl<L, F> FormatTokens<L> for FromFn<F>
where
    L: Lang,
    F: FnOnce(&mut Tokens<L>),
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        (self.f)(tokens);
    }
}