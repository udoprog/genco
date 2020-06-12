use crate::lang::Lang;
use crate::tokens;
use crate::Tokens;

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

impl<T, L> tokens::FormatInto<L> for Quoted<T>
where
    L: Lang,
    T: Into<tokens::ItemStr>,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(tokens::Item::Quoted(self.inner.into()));
    }
}
