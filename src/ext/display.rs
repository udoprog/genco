use crate::lang::Lang;
use crate::tokens;
use crate::Tokens;
use std::fmt;

/// Struct containing a type that implements [Display][fmt::Display] and can be
/// tokenized into a stream.
///
/// This is constructed with the [DisplayExt::display()] function.
///
/// [DisplayExt::display()]: crate::DisplayExt::display()
#[derive(Clone, Copy)]
pub struct Display<'a, T> {
    inner: &'a T,
}

impl<'a, T> Display<'a, T> {
    pub(super) fn new(inner: &'a T) -> Self {
        Self { inner }
    }
}

impl<'a, T, L> tokens::FormatInto<L> for Display<'a, T>
where
    L: Lang,
    T: fmt::Display,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(tokens::Item::Literal(
            self.inner.to_string().into_boxed_str().into(),
        ));
    }
}
