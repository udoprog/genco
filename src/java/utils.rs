use crate::{Cons, Element, IntoTokens, Java, Tokens};

/// Format a block comment, starting with `/**`, and ending in `*/`.
pub struct BlockComment<'el>(pub Vec<Cons<'el>>);

impl<'el> IntoTokens<'el, Java<'el>> for BlockComment<'el> {
    fn into_tokens(self) -> Tokens<'el, Java<'el>> {
        let mut t = Tokens::new();

        if self.0.is_empty() {
            return t;
        }

        t.push("/**");

        for line in self.0 {
            t.push(" * ");
            t.append(line);
        }

        t.push(" */");
        t.push(Element::PushSpacing);

        t
    }
}
