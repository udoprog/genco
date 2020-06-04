use crate::{Cons, ErasedElement, FormatTokens, Lang};

mod tokenize_iter;

pub use self::tokenize_iter::TokenizeIter;

/// Tokenizer for various types.
pub trait Ext {
    /// Trait to provide string quoting through `<stmt>.quoted()`.
    ///
    /// This is used to generated quoted strings, in the language of choice.
    ///
    /// # Examples
    ///
    /// Example showcasing quoted strings when generating Rust.
    ///
    /// ```rust
    /// #![feature(proc_macro_hygiene)]
    /// use genco::prelude::*;
    /// use genco::rust::imported;
    ///
    /// let map = imported("std::collections", "HashMap").qualified();
    ///
    /// let tokens = genco::quote! {
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
    ///        ""
    ///     ],
    ///     tokens.to_file_vec().unwrap(),
    /// );
    /// ```
    fn quoted<'el>(self) -> ErasedElement<'el>
    where
        Self: Into<Cons<'el>>,
    {
        ErasedElement::Quoted(self.into())
    }

    /// Tokenize an iterator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #![feature(proc_macro_hygiene)]
    ///
    /// use genco::{Tokens, quote, Ext as _, Rust};
    /// use rand::Rng;
    ///
    /// use std::fmt;
    ///
    /// # fn main() -> fmt::Result {
    /// // Iterators can be tokenized using `tokenize_iter`, as long as they contain
    /// // something which can be converted into a stream of tokens.
    /// let numbers = (0..10)
    ///     .map(|_| quote!(#(rand::thread_rng().gen::<i16>())#(", ")))
    ///     .chain(Some(quote!(#(rand::thread_rng().gen::<i16>()))))
    ///     .tokenize_iter();
    ///
    /// let tokens: Tokens<Rust> = quote! {
    ///     let data = vec![#numbers];
    /// };
    /// # Ok(())
    /// # }

    fn tokenize_iter<'el, L>(self) -> TokenizeIter<Self>
    where
        Self: Sized + IntoIterator,
        Self::Item: FormatTokens<'el, L>,
        L: Lang,
    {
        TokenizeIter::new(self)
    }
}

impl<T> Ext for T {}
