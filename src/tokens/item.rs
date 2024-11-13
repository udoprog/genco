//! A single element

use alloc::boxed::Box;

use crate::lang::Lang;
use crate::tokens::{FormatInto, ItemStr, Tokens};

/// A single item in a stream of tokens.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Item<L>
where
    L: Lang,
{
    /// A literal item.
    /// Is added as a raw string to the stream of tokens.
    Literal(ItemStr),
    /// A language-specific item.
    Lang(usize, Box<L::Item>),
    /// A language-specific item that is not rendered.
    Register(usize, Box<L::Item>),
    /// Push a new line unless the current line is empty. Will be flushed on
    /// indentation changes.
    Push,
    /// Push a line. Will be flushed on indentation changes.
    Line,
    /// Space between language items. Typically a single space.
    ///
    /// Multiple spacings in sequence are collapsed into one.
    /// A spacing does nothing if at the beginning of a line.
    Space,
    /// Manage indentation.
    ///
    /// An indentation of 0 has no effect.
    Indentation(i16),
    /// Switch to handling input as a quote.
    ///
    /// The argument indicates whether the string contains any interpolated
    /// values.
    ///
    /// The string content is quoted with the language-specific [quoting method].
    /// [quoting method]: Lang::Openquote_string
    OpenQuote(bool),
    /// Close the current quote.
    CloseQuote,
    /// Switch on evaluation. Only valid during string handling.
    OpenEval,
    /// Close evaluation.
    CloseEval,
}

/// Formatting an item is the same as simply adding that item to the token
/// stream.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
/// use genco::tokens::{Item, ItemStr};
///
/// let foo = Item::Literal(ItemStr::Static("foo"));
/// let bar = Item::Literal(ItemStr::Box("bar".into()));
///
/// let result: Tokens = quote!($foo $bar baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
///
/// assert_eq!{
///     vec![
///         Item::Literal(ItemStr::Static("foo")),
///         Item::Space,
///         Item::Literal(ItemStr::Box("bar".into())),
///         Item::Space,
///         Item::Literal(ItemStr::Static("baz")),
///     ] as Vec<Item<()>>,
///     result,
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for Item<L>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(self);
    }
}
