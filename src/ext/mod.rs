//! Extension traits provided by GenCo.

use crate::ItemStr;

mod quoted;

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
    /// #[feature(proc_macro_hygiene)]
    /// use genco::prelude::*;
    ///
    /// let map = rust::imported("std::collections", "HashMap").qualified();
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
