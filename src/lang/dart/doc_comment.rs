use crate::{Dart, FormatTokens, ItemStr, Tokens};

/// Format a doc comment where each line is preceeded by `///`.
/// This struct is created by the [doc_comment][super::doc_comment] function.
pub struct DocComment<T>(pub(super) T);

impl<T> FormatTokens<Dart> for DocComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    fn format_tokens(self, tokens: &mut Tokens<Dart>) {
        for line in self.0 {
            tokens.push();
            tokens.append("/// ");
            tokens.append(line.into());
        }
    }
}
