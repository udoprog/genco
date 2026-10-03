use crate::lang::comment::for_each_line;
use crate::lang::Java;
use crate::tokens::{self, ItemStr};
use crate::Tokens;

/// Format a block comment, starting with `/**`, and ending in `*/`.
///
/// This struct is created by the [block_comment][super::block_comment()] function.
pub struct BlockComment<T>(pub(super) T);

impl<T> tokens::FormatInto<Java> for BlockComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
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
            for_each_line(line.into(), |line| {
                tokens.space();
                tokens.append(tokens::static_literal("*"));
                tokens.space();
                tokens.append(escape_terminator(line));
                tokens.push();
            });
        }

        tokens.space();
        tokens.append("*/");
    }
}

/// `*/` would end the comment early. `*&#47;` keeps the text readable and
/// renders as `*/` in Javadoc.
fn escape_terminator(line: ItemStr) -> ItemStr {
    if line.contains("*/") {
        ItemStr::from(line.replace("*/", "*&#47;"))
    } else {
        line
    }
}
