use crate::{Cons, Csharp, Element, IntoTokens, Tokens};

/// Format a block comment, starting with `/**`, and ending in `*/`.
pub struct BlockComment<'el>(pub Vec<Cons<'el>>);

impl<'el> IntoTokens<'el, Csharp<'el>> for BlockComment<'el> {
    fn into_tokens(self) -> Tokens<'el, Csharp<'el>> {
        let mut t = Tokens::new();

        if self.0.is_empty() {
            return t;
        }

        for line in self.0 {
            t.push("/// ");
            t.append(line);
        }

        t.push(Element::PushSpacing);

        t
    }
}
