use crate::{FormatTokens, ItemStr, Java, Tokens};

/// Format a block comment, starting with `/**`, and ending in `*/`.
///
/// This struct is created by the [block_comment][super::block_comment()] function.
pub struct BlockComment<T>(pub(super) T);

impl<T> FormatTokens<Java> for BlockComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    fn format_tokens(self, tokens: &mut Tokens<Java>) {
        let mut it = self.0.into_iter().peekable();

        if it.peek().is_none() {
            return;
        }

        tokens.push();
        tokens.append(ItemStr::Static("/**"));
        tokens.push();

        for line in it {
            tokens.space();
            tokens.append(ItemStr::Static("*"));
            tokens.space();
            tokens.append(line.into());
            tokens.push();
        }

        tokens.space();
        tokens.append("*/");
    }
}
