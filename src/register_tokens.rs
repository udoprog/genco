//! Converter traits for things that can be converted into tokens.

use super::{Element, Lang, LangBox, Tokens};

/// Helper trait to convert something into a tokens registration.
pub trait RegisterTokens<'el, L>
where
    L: Lang,
{
    /// Convert the type into tokens.
    fn register_tokens(self, tokens: &mut Tokens<'el, L>);
}

impl<'el, T, L: 'el> RegisterTokens<'el, L> for T
where
    T: Into<LangBox<'el, L>>,
    L: Lang,
{
    fn register_tokens(self, tokens: &mut Tokens<'el, L>) {
        tokens.elements.push(Element::Registered(self.into()))
    }
}

macro_rules! impl_register_tokens {
    ($($ty:ident => $var:ident),*) => {
        impl<'el, L: 'el, $($ty,)*> RegisterTokens<'el, L> for ($($ty,)*)
        where
            $($ty: Into<LangBox<'el, L>>,)*
            L: Lang,
        {
            fn register_tokens(self, tokens: &mut Tokens<'el, L>) {
                let ($($var,)*) = self;
                $(tokens.elements.push(Element::Registered($var.into()));)*
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
