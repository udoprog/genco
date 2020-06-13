use crate::lang::Lang;
use crate::tokens::{FormatInto, ItemStr};
use crate::Tokens;

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
/// let map = rust::imported("std::collections", "HashMap");
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
pub fn quoted<T>(inner: T) -> Quoted<T>
where
    T: Into<ItemStr>,
{
    Quoted { inner }
}

/// Struct containing a type that is quoted.
///
/// This is constructed with the [quoted()] function.
#[derive(Clone, Copy)]
pub struct Quoted<T> {
    inner: T,
}

impl<T, L> FormatInto<L> for Quoted<T>
where
    L: Lang,
    T: Into<ItemStr>,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.quoted(self.inner);
    }
}
