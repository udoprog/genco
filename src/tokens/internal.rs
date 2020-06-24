use crate::lang::Lang;
use crate::tokens::{from_fn, FormatInto};

/// Add a language item directly.
///
/// This must only be used by the [decl_lang!] macro.
///
/// [decl_lang!]: crate::decl_lang!
pub fn __lang_item<L>(item: Box<L::Item>) -> impl FormatInto<L>
where
    L: Lang,
{
    from_fn(|t| {
        t.lang_item(item);
    })
}

/// Register a language item directly.
///
/// This must only be used by the [decl_lang!] macro.
///
/// [decl_lang!]: crate::decl_lang!
pub fn __lang_item_register<L>(item: Box<L::Item>) -> impl FormatInto<L>
where
    L: Lang,
{
    from_fn(|t| {
        t.lang_item_register(item);
    })
}
