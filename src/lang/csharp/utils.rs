use crate::{Csharp, Element, FormatTokens, ItemStr, Tokens};

/// Format a block comment, starting with `/**`, and ending in `*/`.
pub struct BlockComment(pub Vec<ItemStr>);

impl FormatTokens<Csharp> for BlockComment {
    fn format_tokens(self, tokens: &mut Tokens<Csharp>) {
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
