use core::fmt;

use alloc::string::ToString;

use crate::lang::Lang;
use crate::tokens::{FormatInto, Item};
use crate::Tokens;

/// Function to build a string literal.
///
/// This is an alternative to manually implementing [tokens::FormatInto], since
/// it can tokenize anything that implements [Display][fmt::Display]
/// directly.
///
/// On the other hand, things implementing [tokens::FormatInto] have access to the
/// full range of the [Tokens] api, allowing it to work more efficiently.
///
/// [tokens::FormatInto]: crate::tokens::FormatInto
/// [Tokens]: crate::Tokens
///
/// # Examples
///
/// Example showcasing quoted strings when generating Rust.
///
/// ```
/// use genco::prelude::*;
/// use std::fmt;
///
/// struct Foo(());
///
/// impl fmt::Display for Foo {
///     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
///         write!(fmt, "Foo")
///     }
/// }
///
/// let map = rust::import("std::collections", "HashMap");
///
/// let foo = Foo(());
///
/// let tokens = quote! {
///     let mut m = $map::<u32, &str>::new();
///     m.insert(0, $(display(&foo)));
/// };
///
/// assert_eq!(
///     vec![
///        "use std::collections::HashMap;",
///        "",
///        "let mut m = HashMap::<u32, &str>::new();",
///        "m.insert(0, Foo);",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn display<T>(inner: T) -> Display<T>
where
    T: fmt::Display,
{
    Display { inner }
}

/// Struct containing a type that implements [Display][fmt::Display] and can be
/// tokenized into a stream.
///
/// This is constructed with the [display()] function.
#[derive(Clone, Copy)]
pub struct Display<T> {
    inner: T,
}

impl<T, L> FormatInto<L> for Display<T>
where
    L: Lang,
    T: fmt::Display,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(Item::Literal(
            self.inner.to_string().into_boxed_str().into(),
        ));
    }
}
