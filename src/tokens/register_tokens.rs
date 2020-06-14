use crate::lang::LangBox;
use crate::tokens::Item;
use crate::Tokens;

mod private {
    pub trait Sealed {}
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
/// let hash_map = rust::imported("std::collections", "HashMap");
/// let btree_map = rust::imported("std::collections", "BTreeMap");
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
pub trait RegisterTokens: private::Sealed {
    /// Convert the type into tokens.
    fn register_tokens(self, tokens: &mut Tokens);
}

impl<T> RegisterTokens for T
where
    T: Into<LangBox>,
{
    fn register_tokens(self, tokens: &mut Tokens) {
        tokens.item(Item::Registered(self.into()))
    }
}

impl<T> self::private::Sealed for T where T: Into<LangBox> {}

macro_rules! impl_register_tokens {
    ($($ty:ident => $var:ident),*) => {
        impl<$($ty,)*> self::private::Sealed for ($($ty,)*)
        where
            $($ty: Into<LangBox>,)*
        {
        }

        impl<$($ty,)*> RegisterTokens for ($($ty,)*)
        where
            $($ty: Into<LangBox>,)*
        {
            fn register_tokens(self, tokens: &mut Tokens) {
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
