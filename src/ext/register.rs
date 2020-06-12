use crate::lang::Lang;
use crate::tokens::{FormatInto, RegisterTokens};
use crate::Tokens;

/// Struct containing a type only intended to be registered.
///
/// This is constructed with the [RegisterExt::register()] function.
///
/// [RegisterExt::register()]: crate::RegisterExt::register()
#[derive(Clone, Copy)]
pub struct Register<T> {
    inner: T,
}

impl<T> Register<T> {
    pub(super) fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T, L> FormatInto<L> for Register<T>
where
    L: Lang,
    T: RegisterTokens<L>,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        self.inner.register_tokens(tokens);
    }
}
