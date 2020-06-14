use crate::tokens;
use crate::Tokens;

/// Format a doc comment where each line is preceeded by `//`.
///
/// This struct is created by the [comment][super::comment()] function.
pub struct Comment<T>(pub(super) T);

impl<T> tokens::FormatInto for Comment<T>
where
    T: IntoIterator,
    T::Item: Into<tokens::ItemStr>,
{
    fn format_into(self, tokens: &mut Tokens) {
        for line in self.0 {
            tokens.push();
            tokens.append(tokens::static_literal("//"));
            tokens.space();
            tokens.append(line.into());
        }
    }
}
