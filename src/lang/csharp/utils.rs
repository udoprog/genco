use crate::{Cons, Csharp, Element, FormatTokens, Tokens};

/// Format a block comment, starting with `/**`, and ending in `*/`.
pub struct BlockComment<'el>(pub Vec<Cons<'el>>);

impl<'el> FormatTokens<'el, Csharp> for BlockComment<'el> {
    fn format_tokens(self, tokens: &mut Tokens<'el, Csharp>) {
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
