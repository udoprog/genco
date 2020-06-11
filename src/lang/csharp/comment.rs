use crate::{Csharp, FormatTokens, ItemStr, Tokens};

/// Format a doc comment where each line is preceeded by `//`.
///
/// This struct is created by the [comment][super::comment()] function.
pub struct Comment<T>(pub(super) T);

impl<T> FormatTokens<Csharp> for Comment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    fn format_tokens(self, tokens: &mut Tokens<Csharp>) {
        for line in self.0 {
            tokens.push();
            tokens.append(ItemStr::Static("//"));
            tokens.space();
            tokens.append(line.into());
        }
    }
}
