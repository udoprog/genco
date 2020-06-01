//! Individual java modifier

use crate::{Custom, Element, IntoTokens, Tokens};
use std::collections::BTreeSet;

/// A Java modifier.
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
    use crate::dart::Dart;
    use crate::tokens::Tokens;

    #[test]
    fn test_vec() {
        use self::Modifier::*;
        let el: Tokens<Dart> = toks![Async, Final].join_spacing();
        let s = el.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("async final"), out);
    }
}
