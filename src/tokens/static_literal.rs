use crate::lang::Lang;
use crate::tokens::{FormatInto, ItemStr};

/// A formatter from a static literal.
///
/// Created from the [static_literal()] function.
#[derive(Debug, Clone, Copy)]
pub struct StaticLiteral {
    literal: &'static str,
}

impl<L> FormatInto<L> for StaticLiteral
where
    L: Lang,
{
    #[inline]
    fn format_into(self, tokens: &mut crate::Tokens<L>) {
        tokens.literal(ItemStr::static_(self.literal));
    }
}

/// A formatter from a static literal.
///
/// This is typically more efficient than using [append()] with a string
/// directly, since it can avoid copying the string.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
/// use genco::tokens;
///
/// let mut tokens = Tokens::<()>::new();
/// tokens.append(tokens::static_literal("hello"));
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// [append()]: crate::Tokens::append()
pub fn static_literal(literal: &'static str) -> StaticLiteral {
    StaticLiteral { literal }
}
