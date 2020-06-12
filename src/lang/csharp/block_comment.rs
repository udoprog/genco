use crate::lang::Csharp;
use crate::tokens::{FormatInto, ItemStr};
use crate::Tokens;

/// Format a doc comment where each line is preceeded by `///`.
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
            tokens.push();
            tokens.append(ItemStr::Static("///"));
            tokens.space();
            tokens.append(line.into());
        }
    }
}
