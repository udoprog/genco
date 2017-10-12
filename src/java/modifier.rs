//! Individual java modifier

use element::Element;
use tokens::Tokens;
use custom::Custom;
use std::collections::BTreeSet;

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
        }
    }
}

impl<'el, C: Custom> From<Vec<Modifier>> for Tokens<'el, C> {
    fn from(value: Vec<Modifier>) -> Self {
        toks![value]
    }
}

impl<'el, C: Custom> From<Vec<Modifier>> for Element<'el, C> {
    fn from(value: Vec<Modifier>) -> Self {
        let out: BTreeSet<&str> = value.iter().map(Modifier::name).collect();
        let out: Vec<&str> = out.into_iter().collect();
        Element::Literal(out.join(" ").into())
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
        let el: Tokens<Java> = vec![Final, Static, Public, Static].into();
        let s = el.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("final public static"), out);
    }
}
