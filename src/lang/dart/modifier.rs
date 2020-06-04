//! Individual java modifier

use crate::{Dart, FormatTokens, Tokens};
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

#[cfg(test)]
mod tests {
    use super::Modifier;
    use crate as genco;
    use crate::dart::Tokens;
    use crate::quote;

    #[test]
    fn test_vec() {
        use self::Modifier::*;
        let el: Tokens = quote!(#(vec![Async, Final]));
        let s = el.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("async final"), out);
    }
}
