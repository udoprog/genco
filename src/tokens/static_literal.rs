use crate::lang;
use crate::tokens;

/// Construct a formatter from a static string.
///
/// This is typically more efficient than using [append()] with a string
/// directly, since it can avoid copying the string.
///
/// # Examples
///
/// ```rust
/// # fn main() -> genco::fmt::Result {
/// use genco::prelude::*;
/// use genco::tokens;
///
/// let mut tokens = Tokens::<()>::new();
/// tokens.append(tokens::static_literal("hello"));
/// # Ok(())
/// # }
/// ```
///
/// [append()]: crate::Tokens::append()
pub fn static_literal<L>(s: &'static str) -> impl tokens::FormatInto<L>
where
    L: lang::Lang,
{
    tokens::from_fn(move |t| {
        t.item(tokens::Item::Literal(tokens::ItemStr::Static(s)));
    })
}
