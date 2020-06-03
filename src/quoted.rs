use crate::{cons::Cons, ErasedElement};
use std::rc::Rc;

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
pub trait Quoted<'el> {
    /// Convert type to quoted element.
    fn quoted(self) -> ErasedElement<'el>;
}

impl<'el> Quoted<'el> for String {
    fn quoted(self) -> ErasedElement<'el> {
        ErasedElement::Quoted(Cons::Rc(Rc::new(self)))
    }
}

impl<'el> Quoted<'el> for &'el str {
    fn quoted(self) -> ErasedElement<'el> {
        ErasedElement::Quoted(Cons::Borrowed(self))
    }
}

impl<'el> Quoted<'el> for Rc<String> {
    fn quoted(self) -> ErasedElement<'el> {
        ErasedElement::Quoted(Cons::Rc(self))
    }
}

impl<'el> Quoted<'el> for Cons<'el> {
    fn quoted(self) -> ErasedElement<'el> {
        ErasedElement::Quoted(self)
    }
}
