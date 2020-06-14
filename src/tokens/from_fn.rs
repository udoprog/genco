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
/// fn comment<L>(s: impl Into<ItemStr>) -> impl FormatInto
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
pub fn from_fn<F>(f: F) -> FromFn<F>
where
    F: FnOnce(&mut Tokens)
{
    FromFn { f }
}

/// A captured function used for formatting tokens.
///
/// Created using [from_fn()].
pub struct FromFn<F> {
    f: F,
}

impl<F> tokens::FormatInto for FromFn<F>
where
    F: FnOnce(&mut Tokens),
{
    fn format_into(self, tokens: &mut Tokens) {
        (self.f)(tokens);
    }
}
