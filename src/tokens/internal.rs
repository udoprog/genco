use alloc::boxed::Box;

use crate::lang::Lang;
use crate::tokens::{from_fn, FormatInto};

/// Add a language item directly.
///
/// This must only be used by the [impl_lang!] macro.
///
/// [impl_lang!]: crate::impl_lang!
#[doc(hidden)]
#[inline]
pub fn __lang_item<L>(item: L::Item) -> impl FormatInto<L>
where
    L: Lang,
{
    from_fn(|t| {
        t.lang_item(Box::new(item));
    })
}

/// Register a language item directly.
///
/// This must only be used by the [impl_lang!] macro.
///
/// [impl_lang!]: crate::impl_lang!
#[doc(hidden)]
#[inline]
pub fn __lang_item_register<L>(item: L::Item) -> impl FormatInto<L>
where
    L: Lang,
{
    from_fn(|t| {
        t.lang_item_register(Box::new(item));
    })
}
