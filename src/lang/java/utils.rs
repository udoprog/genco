use crate::{Cons, Element, FormatTokens, Java, Tokens};

/// Format a block comment, starting with `/**`, and ending in `*/`.
pub struct BlockComment<'el>(pub Vec<Cons<'el>>);

impl<'el> FormatTokens<'el, Java> for BlockComment<'el> {
    fn format_tokens(self, tokens: &mut Tokens<'el, Java>) {
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
