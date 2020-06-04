use crate::{FormatTokens, Lang, Tokens};

/// Tokenizer for an iterator.
pub struct TokenizeIter<I> {
    iter: I,
}

impl<I> TokenizeIter<I> {
    pub(crate) fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<'el, L, I> FormatTokens<'el, L> for crate::TokenizeIter<I>
where
    L: Lang,
    I: IntoIterator,
    I::Item: FormatTokens<'el, L>,
{
    fn format_tokens(self, tokens: &mut Tokens<'el, L>) {
        for element in self.iter {
            tokens.append(element);
        }
    }
}
