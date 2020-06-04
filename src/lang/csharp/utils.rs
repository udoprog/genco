use crate::{quote_in, Csharp, FormatTokens, ItemStr, Tokens};

/// Format a block comment, starting with `/**`, and ending in `*/`.
pub struct BlockComment(pub Vec<ItemStr>);

impl FormatTokens<Csharp> for BlockComment {
    fn format_tokens(self, tokens: &mut Tokens<Csharp>) {
        for line in self.0 {
            quote_in!(tokens => #("///") #line);
            tokens.push();
        }
    }
}
