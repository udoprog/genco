use crate::lang::Dart;
use crate::tokens;
use crate::Tokens;

/// Format a doc comment where each line is preceeded by `///`.
///
/// This struct is created by the [doc_comment][super::doc_comment()] function.
pub struct DocComment<T>(pub(super) T);

impl<T> tokens::FormatInto<Dart> for DocComment<T>
where
    T: IntoIterator,
    T::Item: Into<tokens::ItemStr>,
{
    fn format_into(self, tokens: &mut Tokens<Dart>) {
        for line in self.0 {
            tokens.push();
            tokens.append(tokens::static_literal("///"));
            tokens.space();
            tokens.append(line.into());
        }
    }
}
