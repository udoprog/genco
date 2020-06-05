use crate::{FormatTokens, Item, ItemStr, Lang, Tokens};

/// Struct containing a type that is quoted.
///
/// This is constructed with the [quoted][super::QuotedExt::quoted] function.
#[derive(Clone, Copy)]
pub struct Quoted<T> {
    inner: T,
}

impl<T> Quoted<T> {
    pub(super) fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T, L> FormatTokens<L> for Quoted<T>
where
    L: Lang,
    T: Into<ItemStr>,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.elements.push(Item::Quoted(self.inner.into()));
    }
}
