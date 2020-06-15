use crate::lang::Lang;
use crate::tokens::{from_fn, FormatInto, Item};

/// Function to provide string quoting.
///
/// Note that quoting is applied automatically for literal strings inside of
/// the [quote!] macro, like: `quote!("hello")`.
///
/// This is used to generated quoted strings, in the language of choice.
///
/// # Examples
///
/// Example showcasing quoted strings when generating Rust.
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let map = rust::import("std::collections", "HashMap");
///
/// let tokens = quote! {
///     let mut m = #map::<u32, &str>::new();
///     m.insert(0, #(quoted("hello\" world")));
/// };
///
/// assert_eq!(
///     vec![
///        "use std::collections::HashMap;",
///        "",
///        "let mut m = HashMap::<u32, &str>::new();",
///        "m.insert(0, \"hello\\\" world\");",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok(())
/// # }
/// ```
///
/// [quote!]: macro.quote.html
pub fn quoted<T, L>(inner: T) -> impl FormatInto<L>
where
    T: FormatInto<L>,
    L: Lang,
{
    from_fn(move |t| {
        t.item(Item::OpenQuote(false));
        inner.format_into(t);
        t.item(Item::CloseQuote);
    })
}
