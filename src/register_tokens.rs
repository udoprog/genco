//! Converter traits for things that can be converted into tokens.

use super::{Item, Lang, LangBox, Tokens};

/// Helper trait to convert something into a tokens registration.
pub trait RegisterTokens<L>
where
    L: Lang,
{
    /// Convert the type into tokens.
    fn register_tokens(self, tokens: &mut Tokens<L>);
}

impl<T, L> RegisterTokens<L> for T
where
    T: Into<LangBox<L>>,
    L: Lang,
{
    fn register_tokens(self, tokens: &mut Tokens<L>) {
        tokens.elements.push(Item::Registered(self.into()))
    }
}

macro_rules! impl_register_tokens {
    ($($ty:ident => $var:ident),*) => {
        impl<L, $($ty,)*> RegisterTokens<L> for ($($ty,)*)
        where
            $($ty: Into<LangBox<L>>,)*
            L: Lang,
        {
            fn register_tokens(self, tokens: &mut Tokens<L>) {
                let ($($var,)*) = self;
                $(tokens.elements.push(Item::Registered($var.into()));)*
            }
        }
    }
}

impl_register_tokens!(A => a);
impl_register_tokens!(A => a, B => b);
impl_register_tokens!(A => a, B => b, C => c);
impl_register_tokens!(A => a, B => b, C => c, D => d);
impl_register_tokens!(A => a, B => b, C => c, D => d, E => e);
impl_register_tokens!(A => a, B => b, C => c, D => d, E => e, F => f);
impl_register_tokens!(A => a, B => b, C => c, D => d, E => e, F => f, G => g);
impl_register_tokens!(A => a, B => b, C => c, D => d, E => e, F => f, G => g, H => h);
