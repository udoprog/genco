//! A single element

use core::cmp::Ordering;
use core::fmt;
use core::hash;
use core::mem;

use alloc::boxed::Box;

use crate::lang::Lang;
use crate::tokens::{FormatInto, ItemStr, Tokens};

/// A single item in a stream of tokens.
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

impl<L> fmt::Debug for Item<L>
where
    L: Lang,
    L::Item: core::fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Literal(lit) => f.debug_tuple("Literal").field(lit).finish(),
            Self::Lang(n, item) => f.debug_tuple("Lang").field(n).field(item).finish(),
            Self::Register(n, item) => f.debug_tuple("Register").field(n).field(item).finish(),
            Self::Push => write!(f, "Push"),
            Self::Line => write!(f, "Line"),
            Self::Space => write!(f, "Space"),
            Self::Indentation(n) => f.debug_tuple("Indentation").field(n).finish(),
            Self::OpenQuote(b) => f.debug_tuple("OpenQuote").field(b).finish(),
            Self::CloseQuote => write!(f, "CloseQuote"),
            Self::OpenEval => write!(f, "OpenEval"),
            Self::CloseEval => write!(f, "CloseEval"),
        }
    }
}

impl<L> Clone for Item<L>
where
    L: Lang,
    L::Item: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::Literal(lit) => Self::Literal(lit.clone()),
            Self::Lang(n, item) => Self::Lang(*n, item.clone()),
            Self::Register(n, item) => Self::Register(*n, item.clone()),
            Self::Push => Self::Push,
            Self::Line => Self::Line,
            Self::Space => Self::Space,
            Self::Indentation(n) => Self::Indentation(*n),
            Self::OpenQuote(b) => Self::OpenQuote(*b),
            Self::CloseQuote => Self::CloseQuote,
            Self::OpenEval => Self::OpenEval,
            Self::CloseEval => Self::CloseEval,
        }
    }
}

impl<L, U> PartialEq<Item<U>> for Item<L>
where
    L: Lang,
    U: Lang,
    L::Item: PartialEq<U::Item>,
{
    #[inline]
    fn eq(&self, other: &Item<U>) -> bool {
        match (self, other) {
            (Self::Literal(a), Item::Literal(b)) => a == b,
            (Self::Lang(na, a), Item::Lang(nb, b)) => na == nb && **a == **b,
            (Self::Register(na, a), Item::Register(nb, b)) => na == nb && **a == **b,
            (Self::Push, Item::Push) => true,
            (Self::Line, Item::Line) => true,
            (Self::Space, Item::Space) => true,
            (Self::Indentation(na), Item::Indentation(nb)) => na == nb,
            (Self::OpenQuote(ba), Item::OpenQuote(bb)) => ba == bb,
            (Self::CloseQuote, Item::CloseQuote) => true,
            (Self::OpenEval, Item::OpenEval) => true,
            (Self::CloseEval, Item::CloseEval) => true,
            _ => false,
        }
    }
}

impl<L> Eq for Item<L>
where
    L: Lang,
    L::Item: Eq,
{
}

impl<L, U> PartialOrd<Item<U>> for Item<L>
where
    L: Lang,
    U: Lang,
    L::Item: PartialOrd<U::Item>,
{
    #[inline]
    fn partial_cmp(&self, other: &Item<U>) -> Option<Ordering> {
        match (self, other) {
            (Self::Literal(a), Item::Literal(b)) => a.partial_cmp(b),
            (Self::Lang(na, a), Item::Lang(nb, b)) => match na.cmp(nb) {
                Ordering::Equal => PartialOrd::partial_cmp(a.as_ref(), b.as_ref()),
                o => Some(o),
            },
            (Self::Register(na, a), Item::Register(nb, b)) => match na.cmp(nb) {
                Ordering::Equal => PartialOrd::partial_cmp(a.as_ref(), b.as_ref()),
                o => Some(o),
            },
            (Self::Push, Item::Push) => Some(Ordering::Equal),
            (Self::Line, Item::Line) => Some(Ordering::Equal),
            (Self::Space, Item::Space) => Some(Ordering::Equal),
            (Self::Indentation(na), Item::Indentation(nb)) => na.partial_cmp(nb),
            (Self::OpenQuote(ba), Item::OpenQuote(bb)) => ba.partial_cmp(bb),
            (Self::CloseQuote, Item::CloseQuote) => Some(Ordering::Equal),
            (Self::OpenEval, Item::OpenEval) => Some(Ordering::Equal),
            (Self::CloseEval, Item::CloseEval) => Some(Ordering::Equal),
            _ => None,
        }
    }
}

impl<L> Ord for Item<L>
where
    L: Lang,
    L::Item: Ord,
{
    #[inline]
    fn cmp(&self, other: &Item<L>) -> Ordering {
        match (self, other) {
            (Self::Literal(a), Item::Literal(b)) => a.cmp(b),
            (Self::Lang(na, a), Item::Lang(nb, b)) => match na.cmp(nb) {
                Ordering::Equal => Ord::cmp(a.as_ref(), b.as_ref()),
                o => o,
            },
            (Self::Register(na, a), Item::Register(nb, b)) => match na.cmp(nb) {
                Ordering::Equal => Ord::cmp(a.as_ref(), b.as_ref()),
                o => o,
            },
            (Self::Push, Item::Push) => Ordering::Equal,
            (Self::Line, Item::Line) => Ordering::Equal,
            (Self::Space, Item::Space) => Ordering::Equal,
            (Self::Indentation(na), Item::Indentation(nb)) => na.cmp(nb),
            (Self::OpenQuote(ba), Item::OpenQuote(bb)) => ba.cmp(bb),
            (Self::CloseQuote, Item::CloseQuote) => Ordering::Equal,
            (Self::OpenEval, Item::OpenEval) => Ordering::Equal,
            (Self::CloseEval, Item::CloseEval) => Ordering::Equal,
            (Self::Literal(_), _) => Ordering::Less,
            (_, Self::Literal(_)) => Ordering::Greater,
            (Self::Lang(_, _), _) => Ordering::Less,
            (_, Self::Lang(_, _)) => Ordering::Greater,
            (Self::Register(_, _), _) => Ordering::Less,
            (_, Self::Register(_, _)) => Ordering::Greater,
            (Self::Push, _) => Ordering::Less,
            (_, Self::Push) => Ordering::Greater,
            (Self::Line, _) => Ordering::Less,
            (_, Self::Line) => Ordering::Greater,
            (Self::Space, _) => Ordering::Less,
            (_, Self::Space) => Ordering::Greater,
            (Self::Indentation(_), _) => Ordering::Less,
            (_, Self::Indentation(_)) => Ordering::Greater,
            (Self::OpenQuote(_), _) => Ordering::Less,
            (_, Self::OpenQuote(_)) => Ordering::Greater,
            (Self::CloseQuote, _) => Ordering::Less,
            (_, Self::CloseQuote) => Ordering::Greater,
            (Self::OpenEval, _) => Ordering::Less,
            (_, Self::OpenEval) => Ordering::Greater,
        }
    }
}

impl<L> hash::Hash for Item<L>
where
    L: Lang,
    L::Item: hash::Hash,
{
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        mem::discriminant(self).hash(state);

        match self {
            Item::Literal(item_str) => {
                item_str.hash(state);
            }
            Item::Lang(n, item) => {
                n.hash(state);
                item.hash(state);
            }
            Item::Register(n, item) => {
                n.hash(state);
                item.hash(state);
            }
            Item::Push => {}
            Item::Line => {}
            Item::Space => {}
            Item::Indentation(n) => {
                n.hash(state);
            }
            Item::OpenQuote(n) => {
                n.hash(state);
            }
            Item::CloseQuote => {}
            Item::OpenEval => {}
            Item::CloseEval => {}
        }
    }
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
