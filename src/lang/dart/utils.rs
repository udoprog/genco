use crate::dart::Tokens;
use crate::{Cons, Dart, IntoTokens};

/// Format a doc comment where each line is preceeded by `///`.
pub struct DocComment<'el>(pub Vec<Cons<'el>>);

impl<'el> IntoTokens<'el, Dart<'el>> for DocComment<'el> {
    fn into_tokens(self, tokens: &mut Tokens<'el>) {
        if self.0.is_empty() {
            return;
        }

        for line in self.0 {
            tokens.push("/// ");
            tokens.append(line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_comment() {
        let toks = toks![
            DocComment(vec![Cons::from("Foo")]),
            DocComment(vec![]),
            DocComment(vec![]),
            DocComment(vec![]),
            DocComment(vec![Cons::from("Bar")]),
        ];

        let expected = vec!["/// Foo", "/// Bar", ""];

        assert_eq!(
            Ok(expected.join("\n").as_str()),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
