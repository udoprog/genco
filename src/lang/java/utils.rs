use crate::{Cons, Element, IntoTokens, Java, Tokens};

/// Format a block comment, starting with `/**`, and ending in `*/`.
pub struct BlockComment<'el>(pub Vec<Cons<'el>>);

impl<'el> IntoTokens<'el, Java<'el>> for BlockComment<'el> {
    fn into_tokens(self, tokens: &mut Tokens<'el, Java<'el>>) {
        if self.0.is_empty() {
            return;
        }

        tokens.push("/**");

        for line in self.0 {
            tokens.push(" * ");
            tokens.append(line);
        }

        tokens.push(" */");
        tokens.push(Element::PushSpacing);
    }
}
