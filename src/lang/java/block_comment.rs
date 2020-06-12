use crate::lang::Java;
use crate::tokens;
use crate::Tokens;

/// Format a block comment, starting with `/**`, and ending in `*/`.
///
/// This struct is created by the [block_comment][super::block_comment()] function.
pub struct BlockComment<T>(pub(super) T);

impl<T> tokens::FormatInto<Java> for BlockComment<T>
where
    T: IntoIterator,
    T::Item: Into<tokens::ItemStr>,
{
    fn format_into(self, tokens: &mut Tokens<Java>) {
        let mut it = self.0.into_iter().peekable();

        if it.peek().is_none() {
            return;
        }

        tokens.push();
        tokens.append(tokens::static_literal("/**"));
        tokens.push();

        for line in it {
            tokens.space();
            tokens.append(tokens::static_literal("*"));
            tokens.space();
            tokens.append(line.into());
            tokens.push();
        }

        tokens.space();
        tokens.append("*/");
    }
}
