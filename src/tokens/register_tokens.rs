use crate::lang::{Lang, LangBox};
use crate::tokens::Item;
use crate::Tokens;

mod private {
    pub trait Sealed<L> {}
}

/// Helper trait to convert something into a tokens registration.
///
/// Thanks to this, we can provide a flexible number of arguments to
/// [register()], like a collection of tuples.
///
/// This trait is *sealed* to prevent downstream implementations.
///
/// [register()]: crate::Tokens::register()
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let mut tokens = rust::Tokens::new();
///
/// let hash_map = rust::import("std::collections", "HashMap");
/// let btree_map = rust::import("std::collections", "BTreeMap");
///
/// tokens.register((hash_map, btree_map));
///
/// assert_eq!(
///     vec![
///         "use std::collections::{BTreeMap, HashMap};",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok(())
/// # }
/// ```
pub trait RegisterTokens<L>: private::Sealed<L>
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
        tokens.item(Item::Registered(self.into()))
    }
}

impl<T, L> self::private::Sealed<L> for T
where
    T: Into<LangBox<L>>,
    L: Lang,
{
}

macro_rules! impl_register_tokens {
    ($($ty:ident => $var:ident),*) => {
        impl<L, $($ty,)*> self::private::Sealed<L> for ($($ty,)*)
        where
            $($ty: Into<LangBox<L>>,)*
            L: Lang,
        {
        }

        impl<L, $($ty,)*> RegisterTokens<L> for ($($ty,)*)
        where
            $($ty: Into<LangBox<L>>,)*
            L: Lang,
        {
            fn register_tokens(self, tokens: &mut Tokens<L>) {
                let ($($var,)*) = self;
                $(tokens.item(Item::Registered($var.into()));)*
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
