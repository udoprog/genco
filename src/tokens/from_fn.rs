use crate::lang::Lang;
use crate::tokens;
use crate::Tokens;

/// Construct a formatter from a function.
///
/// # Examples
///
/// ```rust
/// # fn main() -> genco::fmt::Result {
/// use genco::quote_in;
/// use genco::lang::Lang;
/// use genco::tokens::{ItemStr, FormatInto, from_fn, static_literal};
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

impl<L, F> tokens::FormatInto<L> for FromFn<F>
where
    L: Lang,
    F: FnOnce(&mut Tokens<L>),
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        (self.f)(tokens);
    }
}
