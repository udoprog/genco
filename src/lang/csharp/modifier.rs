//! Individual C# modifier

use crate::{Csharp, FormatTokens, Tokens};
use std::collections::BTreeSet;

/// A Csharp modifier.
///
/// A vector of modifiers have a custom implementation, allowing them to be
/// formatted with a spacing between them in the language-recommended order.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
/// use csharp::Modifier::*;
///
/// let toks: csharp::Tokens = quote!(#(vec![Static, Public]));
///
/// assert_eq!("public static", toks.to_string().unwrap());
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Modifier {
    /// public
    Public,
    /// private
    Private,
    /// internal
    Internal,
    /// protected
    Protected,
    /// abstract
    Abstract,
    /// async
    Async,
    /// const
    Const,
    /// event
    Event,
    /// extern
    Extern,
    /// new
    New,
    /// override
    Override,
    /// partial
    Partial,
    /// readonly
    Readonly,
    /// sealed
    Sealed,
    /// static
    Static,
    /// unsafe
    Unsafe,
    /// virtual
    Virtual,
    /// volatile
    Volatile,
}

impl Modifier {
    /// Get the name of the modifier.
    pub fn name(&self) -> &'static str {
        use self::Modifier::*;

        match *self {
            Public => "public",
            Private => "private",
            Internal => "internal",
            Protected => "protected",
            Abstract => "abstract",
            Async => "async",
            Const => "const",
            Event => "event",
            Extern => "extern",
            New => "new",
            Override => "override",
            Partial => "partial",
            Readonly => "readonly",
            Sealed => "sealed",
            Static => "static",
            Unsafe => "unsafe",
            Virtual => "virtual",
            Volatile => "volatile",
        }
    }
}

impl FormatTokens<Csharp> for Modifier {
    fn format_tokens(self, tokens: &mut Tokens<Csharp>) {
        tokens.append(self.name());
    }
}

impl FormatTokens<Csharp> for Vec<Modifier> {
    fn format_tokens(self, tokens: &mut Tokens<Csharp>) {
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
