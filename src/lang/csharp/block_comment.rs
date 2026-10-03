use crate::lang::comment::for_each_line;
use crate::lang::Csharp;
use crate::tokens::{FormatInto, ItemStr};
use crate::Tokens;

/// Format an XML doc comment where each line is preceeded by `///`.
///
/// Despite its name, this does not produce a `/* */` block comment.
///
/// This struct is created by the [block_comment][super::block_comment()] function.
pub struct BlockComment<T>(pub(super) T);

impl<T> FormatInto<Csharp> for BlockComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    fn format_into(self, tokens: &mut Tokens<Csharp>) {
        for line in self.0 {
            for_each_line(line.into(), |line| {
                tokens.push();
                tokens.append(ItemStr::static_("///"));
                tokens.space();
                tokens.append(line);
            });
        }
    }
}
