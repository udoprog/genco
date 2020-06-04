//! Individual C# modifier

use crate::{Csharp, FormatTokens, Tokens};
use std::collections::BTreeSet;

/// A Csharp modifier.
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

#[cfg(test)]
mod tests {
    use super::Modifier;
    use crate as genco;
    use crate::{quote, Csharp, Tokens};

    #[test]
    fn test_vec() {
        use self::Modifier::*;
        let el: Tokens<Csharp> = quote!(#(vec![Static, Public]));
        let s = el.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public static"), out);
    }
}
