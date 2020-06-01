use crate::{Cons, Dart, Element, Tokens};

/// Format a doc comment where each line is preceeded by `///`.
pub struct DocComment<'el>(pub Vec<Cons<'el>>);

impl<'el> From<DocComment<'el>> for Element<'el, Dart<'el>> {
    fn from(value: DocComment<'el>) -> Element<'el, Dart<'el>> {
        let mut t = Tokens::new();

        if value.0.is_empty() {
            return Element::None;
        }

        for line in value.0 {
            t.push("/// ");
            t.append(line);
        }

        Element::from(t)
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
