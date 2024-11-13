use crate::lang::Lang;
use crate::tokens;
use crate::Tokens;

/// Construct a [FormatInto][crate::tokens::FormatInto] implementation from a
/// function.
///
/// # Examples
///
/// ```
/// use genco::{quote, quote_in};
/// use genco::lang::{Lang, Rust};
/// use genco::tokens::{ItemStr, FormatInto, Tokens, from_fn, static_literal};
///
/// fn comment(s: impl Into<ItemStr> + Copy) -> impl FormatInto<Rust> + Copy {
///     from_fn(move |tokens| {
///         let s = s.into();
///         quote_in!(*tokens => $(static_literal("//")) #s);
///     })
/// }
///
/// let c = comment("hello world");
/// let _: Tokens<Rust> = quote!($c $['\n'] $c);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[inline]
pub fn from_fn<F, L>(f: F) -> FromFn<F>
where
    F: FnOnce(&mut Tokens<L>),
    L: Lang,
{
    FromFn { f }
}

/// A captured function used for formatting tokens.
///
/// Constructed using [from_fn()] or the [quote_fn!][crate::quote_fn] macro.
#[derive(Clone, Copy)]
pub struct FromFn<F> {
    f: F,
}

impl<L, F> tokens::FormatInto<L> for FromFn<F>
where
    L: Lang,
    F: FnOnce(&mut Tokens<L>),
{
    #[inline]
    fn format_into(self, tokens: &mut Tokens<L>) {
        (self.f)(tokens);
    }
}
