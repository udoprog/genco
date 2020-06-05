//! Individual java modifier

use crate::java::Tokens;
use crate::{FormatTokens, Java};
use std::collections::BTreeSet;

/// A Java modifier.
///
/// A vector of modifiers have a custom implementation, allowing them to be
/// formatted with a spacing between them in the language-recommended order.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
/// use java::Modifier::*;
///
/// let toks: java::Tokens = quote!(#(vec![Public, Final, Static]));
///
/// assert_eq!("public static final", toks.to_string().unwrap());
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Modifier {
    /// default
    Default,
    /// public
    Public,
    /// protected
    Protected,
    /// private
    Private,
    /// abstract
    Abstract,
    /// static
    Static,
    /// final
    Final,
    /// Native
    Native,
}

impl Modifier {
    /// Get the name of the modifier.
    pub fn name(&self) -> &'static str {
        use self::Modifier::*;

        match *self {
            Default => "default",
            Public => "public",
            Protected => "protected",
            Private => "private",
            Abstract => "abstract",
            Static => "static",
            Final => "final",
            Native => "native",
        }
    }
}

impl FormatTokens<Java> for Modifier {
    fn format_tokens(self, tokens: &mut Tokens) {
        tokens.append(self.name());
    }
}

impl FormatTokens<Java> for Vec<Modifier> {
    fn format_tokens(self, tokens: &mut Tokens) {
        let mut it = self.into_iter().collect::<BTreeSet<_>>().into_iter();

        if let Some(modifier) = it.next() {
            modifier.format_tokens(tokens);
        }

        for modifier in it {
            tokens.spacing();
            modifier.format_tokens(tokens);
        }
    }
}
