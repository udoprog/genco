//! Extension traits for working with genco.

use crate::ItemStr;
use std::fmt;

mod display;
mod quoted;

pub use self::display::Display;
pub use self::quoted::Quoted;

/// Tokenizer for various types.
pub trait QuotedExt {
    /// Trait to provide string quoting through `<stmt>.quoted()`.
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
    /// let map = rust::imported("std::collections", "HashMap");
    ///
    /// let tokens = quote! {
    ///     let mut m = #map::<u32, &str>::new();
    ///     m.insert(0, #("hello\" world".quoted()));
    /// };
    ///
    /// assert_eq!(
    ///    vec![
    ///        "use std::collections::HashMap;",
    ///        "",
    ///        "let mut m = HashMap::<u32, &str>::new();",
    ///        "m.insert(0, \"hello\\\" world\");",
    ///     ],
    ///     tokens.to_file_vec().unwrap(),
    /// );
    /// ```
    fn quoted(self) -> Quoted<Self>
    where
        Self: Into<ItemStr>,
    {
        Quoted::new(self)
    }
}

impl<T> QuotedExt for T where T: Into<ItemStr> {}

/// Tokenizer for anything that implements display.
pub trait DisplayExt {
    /// Trait to provide string quoting through `<stmt>.display()`.
    ///
    /// This is an alternative to manually implementing [FormatTokens], since it
    /// can tokenize anything that implements [Display][fmt::Display] directly.
    ///
    /// On the other hand, things implementing [FormatTokens] have access to the
    /// full range of the [Tokens] api, allowing it to work more efficiently.
    ///
    /// [FormatTokens]: crate::FormatTokens
    /// [Tokens]: crate::Tokens
    ///
    /// # Examples
    ///
    /// Example showcasing quoted strings when generating Rust.
    ///
    /// ```rust
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
    /// let map = rust::imported("std::collections", "HashMap");
    ///
    /// let foo = Foo(());
    ///
    /// let tokens = quote! {
    ///     let mut m = #map::<u32, &str>::new();
    ///     m.insert(0, #(foo.display()));
    /// };
    ///
    /// assert_eq!(
    ///    vec![
    ///        "use std::collections::HashMap;",
    ///        "",
    ///        "let mut m = HashMap::<u32, &str>::new();",
    ///        "m.insert(0, Foo);",
    ///     ],
    ///     tokens.to_file_vec().unwrap(),
    /// );
    /// ```
    fn display(&self) -> Display<'_, Self>
    where
        Self: Sized + fmt::Display,
    {
        Display::new(self)
    }
}

impl<T> DisplayExt for T where T: fmt::Display {}
