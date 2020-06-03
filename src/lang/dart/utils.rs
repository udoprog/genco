use crate::{Cons, Dart, FormatTokens, Tokens};

/// Format a doc comment where each line is preceeded by `///`.
pub struct DocComment<'el>(pub Vec<Cons<'el>>);

impl<'el> FormatTokens<'el, Dart> for DocComment<'el> {
    fn format_tokens(self, tokens: &mut Tokens<'el, Dart>) {
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
            toks.to_file_string().as_ref().map(|s| s.as_str())
        );
    }
}
