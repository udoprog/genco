use crate::lang::Lang;
use crate::tokens::{FormatInto, Item, Tokens};

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
/// ```
/// use genco::prelude::*;
///
/// let map = rust::import("std::collections", "HashMap");
///
/// let tokens = quote! {
///     let mut m = $map::<u32, &str>::new();
///     m.insert(0, $(quoted("hello\" world")));
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
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// # Example: A quote inside a quote
///
/// Note that this requires extra buffering to occur when formatting the token stream.
///
/// ```
/// use genco::prelude::*;
///
/// let tokens: python::Tokens = quote!($[str](Hello $[const](quoted("World 😊"))));
///
/// assert_eq!(
///     "\"Hello \\\"World \\U0001f60a\\\"\"",
///     tokens.to_string()?,
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// [quote!]: macro.quote.html
pub fn quoted<T>(inner: T) -> QuotedFn<T> {
    QuotedFn { inner }
}

/// Struct containing a type that is quoted.
///
/// This is constructed with the [quoted()] function.
#[derive(Clone, Copy, Debug)]
pub struct QuotedFn<T> {
    inner: T,
}

impl<T, L> FormatInto<L> for QuotedFn<T>
where
    L: Lang,
    T: FormatInto<L>,
{
    fn format_into(self, t: &mut Tokens<L>) {
        t.item(Item::OpenQuote(false));
        self.inner.format_into(t);
        t.item(Item::CloseQuote);
    }
}
