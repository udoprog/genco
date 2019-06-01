//! Individual java modifier

use std::collections::BTreeSet;
use {Custom, Element, IntoTokens, Tokens};

/// A Java modifier.
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
    Native
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
            Native => "native"
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
    use java::Java;
    use tokens::Tokens;

    #[test]
    fn test_vec() {
        use self::Modifier::*;
        let el: Tokens<Java> = toks![Public, Static, Final].join_spacing();
        let s = el.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public static final"), out);
    }
}
