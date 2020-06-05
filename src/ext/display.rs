use crate::{FormatTokens, Item, Lang, Tokens};
use std::fmt;

/// Struct containing a type that implements [Display][fmt::Display] and can be
/// tokenized into a stream.
///
/// This is constructed with the [display][super::Display::display] function.
#[derive(Clone, Copy)]
pub struct Display<'a, T> {
    inner: &'a T,
}

impl<'a, T> Display<'a, T> {
    pub(super) fn new(inner: &'a T) -> Self {
        Self { inner }
    }
}

impl<'a, T, L> FormatTokens<L> for Display<'a, T>
where
    L: Lang,
    T: fmt::Display,
{
    fn format_tokens(self, tokens: &mut Tokens<L>) {
        tokens.elements.push(Item::Literal(
            self.inner.to_string().into_boxed_str().into(),
        ));
    }
}
