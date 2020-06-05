//! Individual java modifier

use crate::{Dart, FormatTokens, Tokens};
use std::collections::BTreeSet;

/// A Dart modifier.
///
/// A vector of modifiers have a custom implementation, allowing them to be
/// formatted with a spacing between them in the language-recommended order.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
/// use dart::Modifier::*;
///
/// let toks: dart::Tokens = quote!(#(vec![Final, Async]));
///
/// assert_eq!("async final", toks.to_string().unwrap());
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Modifier {
    /// async
    Async,
    /// final
    Final,
}

impl Modifier {
    /// Get the name of the modifier.
    pub fn name(&self) -> &'static str {
        use self::Modifier::*;

        match *self {
            Async => "async",
            Final => "final",
        }
    }
}

impl FormatTokens<Dart> for Vec<Modifier> {
    fn format_tokens(self, tokens: &mut Tokens<Dart>) {
        let mut it = self.into_iter().collect::<BTreeSet<_>>().into_iter();

        if let Some(modifier) = it.next() {
            tokens.append(modifier.name());
        }

        for modifier in it {
            tokens.spacing();
            tokens.append(modifier.name());
        }
    }
}
