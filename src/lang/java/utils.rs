use crate::{FormatTokens, Item, ItemStr, Java, Tokens};

/// Format a block comment, starting with `/**`, and ending in `*/`.
pub struct BlockComment(pub Vec<ItemStr>);

impl FormatTokens<Java> for BlockComment {
    fn format_tokens(self, tokens: &mut Tokens<Java>) {
        if self.0.is_empty() {
            return;
        }

        tokens.push("/**");

        for line in self.0 {
            tokens.push(" * ");
            tokens.append(line);
        }

        tokens.push(" */");
        tokens.push(Item::PushSpacing);
    }
}
