use crate::csharp::Tokens;
use crate::{Cons, Csharp, Element, FormatTokens};

/// Format a block comment, starting with `/**`, and ending in `*/`.
pub struct BlockComment<'el>(pub Vec<Cons<'el>>);

impl<'el> FormatTokens<'el, Csharp<'el>> for BlockComment<'el> {
    fn into_tokens(self, tokens: &mut Tokens<'el>) {
        if self.0.is_empty() {
            return;
        }

        for line in self.0 {
            tokens.push("/// ");
            tokens.append(line);
        }

        tokens.push(Element::PushSpacing);
    }
}
