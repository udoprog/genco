//! Individual C# modifier

use {Custom, Element, IntoTokens, Tokens};
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

impl<'el, C: Custom> From<Modifier> for Element<'el, C> {
    fn from(value: Modifier) -> Self {
        value.name().into()
    }
}

impl<'el, C: Custom> IntoTokens<'el, C> for Vec<Modifier> {
    fn into_tokens(self) -> Tokens<'el, C> {
        self.into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(Element::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Modifier;
    use csharp::Csharp;
    use tokens::Tokens;

    #[test]
    fn test_vec() {
        use self::Modifier::*;
        let el: Tokens<Csharp> = toks![Public, Static].join_spacing();
        let s = el.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public static"), out);
    }
}
