//! A single element

use core::cmp::Ordering;
use core::fmt;
use core::hash;
use core::mem;

use alloc::boxed::Box;

use crate::lang::Lang;
use crate::tokens::{FormatInto, ItemStr, Tokens};

pub(crate) enum ItemKind<L>
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
    OpenQuote(bool),
    /// Close the last quote.
    CloseQuote,
    /// Switch on evaluation. Only valid during string handling.
    OpenEval,
    /// Close evaluation.
    CloseEval,
}

/// A single item in a stream of tokens.
#[non_exhaustive]
pub struct Item<L>
where
    L: Lang,
{
    pub(crate) kind: ItemKind<L>,
}

impl<L> Item<L>
where
    L: Lang,
{
    /// Construct a new item based on a kind.
    #[inline]
    pub(crate) const fn new(kind: ItemKind<L>) -> Self {
        Self { kind }
    }

    /// Shorthand for constructing a new static literal item.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::{Item, ItemStr};
    /// use genco::lang::Rust;
    ///
    /// let a = Item::<Rust>::static_("hello");
    /// let b = Item::<Rust>::literal("hello".into());
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn static_(literal: &'static str) -> Self {
        Self::new(ItemKind::Literal(ItemStr::static_(literal)))
    }

    /// Get the kind of this item.
    #[inline]
    pub(crate) fn kind(&self) -> &ItemKind<L> {
        &self.kind
    }

    /// Construct a new push item.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::Item;
    /// use genco::lang::Rust;
    ///
    /// let a = Item::<Rust>::push();
    /// let b = Item::<Rust>::push();
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn push() -> Self {
        Self::new(ItemKind::Push)
    }

    /// Construct a new line item.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::Item;
    /// use genco::lang::Rust;
    ///
    /// let a = Item::<Rust>::line();
    /// let b = Item::<Rust>::line();
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn line() -> Self {
        Self::new(ItemKind::Line)
    }

    /// Construct a new space item.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::Item;
    /// use genco::lang::Rust;
    ///
    /// let a = Item::<Rust>::space();
    /// let b = Item::<Rust>::space();
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn space() -> Self {
        Self::new(ItemKind::Space)
    }

    /// Construct a new indentation item.
    #[inline]
    pub(crate) const fn indentation(n: i16) -> Self {
        Self::new(ItemKind::Indentation(n))
    }

    /// Construct a quote open.
    ///
    /// Switches to handling input as a quote. The argument indicates whether
    /// the string contains any interpolated values. The string content is
    /// quoted with the language-specific [quoting method].
    ///
    /// [quoting method]: Lang::open_quote
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::Item;
    /// use genco::lang::Rust;
    ///
    /// let a = Item::<Rust>::open_quote(true);
    /// let b = Item::<Rust>::open_quote(false);
    ///
    /// assert_eq!(a, a);
    /// assert_ne!(a, b);
    /// ```
    #[inline]
    pub const fn open_quote(is_interpolated: bool) -> Self {
        Self::new(ItemKind::OpenQuote(is_interpolated))
    }

    /// Construct a quote close.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::Item;
    /// use genco::lang::Rust;
    ///
    /// let a = Item::<Rust>::close_quote();
    /// let b = Item::<Rust>::close_quote();
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn close_quote() -> Self {
        Self::new(ItemKind::CloseQuote)
    }

    /// Construct a new eval open.
    #[inline]
    pub(crate) const fn open_eval() -> Self {
        Self::new(ItemKind::OpenEval)
    }

    /// Construct a new eval close.
    #[inline]
    pub(crate) const fn close_eval() -> Self {
        Self::new(ItemKind::CloseEval)
    }

    /// Construct a new literal item.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::tokens::{Item, ItemStr};
    /// use genco::lang::Rust;
    ///
    /// let a = Item::<Rust>::static_("hello");
    /// let b = Item::<Rust>::literal("hello".into());
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline]
    pub const fn literal(lit: ItemStr) -> Self {
        Self::new(ItemKind::Literal(lit))
    }

    /// Construct a new language-specific item.
    #[inline]
    pub(crate) const fn lang(n: usize, item: Box<L::Item>) -> Self {
        Self::new(ItemKind::Lang(n, item))
    }

    /// Construct a new language-specific register item.
    #[inline]
    pub(crate) const fn register(n: usize, item: Box<L::Item>) -> Self {
        Self::new(ItemKind::Register(n, item))
    }
}

impl<L> fmt::Debug for Item<L>
where
    L: Lang,
    L::Item: core::fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.kind {
            ItemKind::Literal(lit) => f.debug_tuple("Literal").field(lit).finish(),
            ItemKind::Lang(n, item) => f.debug_tuple("Lang").field(n).field(item).finish(),
            ItemKind::Register(n, item) => f.debug_tuple("Register").field(n).field(item).finish(),
            ItemKind::Push => write!(f, "Push"),
            ItemKind::Line => write!(f, "Line"),
            ItemKind::Space => write!(f, "Space"),
            ItemKind::Indentation(n) => f.debug_tuple("Indentation").field(n).finish(),
            ItemKind::OpenQuote(b) => f.debug_tuple("OpenQuote").field(b).finish(),
            ItemKind::CloseQuote => write!(f, "CloseQuote"),
            ItemKind::OpenEval => write!(f, "OpenEval"),
            ItemKind::CloseEval => write!(f, "CloseEval"),
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
        let kind = match &self.kind {
            ItemKind::Literal(lit) => ItemKind::Literal(lit.clone()),
            ItemKind::Lang(n, item) => ItemKind::Lang(*n, item.clone()),
            ItemKind::Register(n, item) => ItemKind::Register(*n, item.clone()),
            ItemKind::Push => ItemKind::Push,
            ItemKind::Line => ItemKind::Line,
            ItemKind::Space => ItemKind::Space,
            ItemKind::Indentation(n) => ItemKind::Indentation(*n),
            ItemKind::OpenQuote(b) => ItemKind::OpenQuote(*b),
            ItemKind::CloseQuote => ItemKind::CloseQuote,
            ItemKind::OpenEval => ItemKind::OpenEval,
            ItemKind::CloseEval => ItemKind::CloseEval,
        };

        Self { kind }
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
        match (&self.kind, &other.kind) {
            (ItemKind::Literal(a), ItemKind::Literal(b)) => a == b,
            (ItemKind::Lang(na, a), ItemKind::Lang(nb, b)) => na == nb && **a == **b,
            (ItemKind::Register(na, a), ItemKind::Register(nb, b)) => na == nb && **a == **b,
            (ItemKind::Push, ItemKind::Push) => true,
            (ItemKind::Line, ItemKind::Line) => true,
            (ItemKind::Space, ItemKind::Space) => true,
            (ItemKind::Indentation(na), ItemKind::Indentation(nb)) => na == nb,
            (ItemKind::OpenQuote(ba), ItemKind::OpenQuote(bb)) => ba == bb,
            (ItemKind::CloseQuote, ItemKind::CloseQuote) => true,
            (ItemKind::OpenEval, ItemKind::OpenEval) => true,
            (ItemKind::CloseEval, ItemKind::CloseEval) => true,
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
        match (&self.kind, &other.kind) {
            (ItemKind::Literal(a), ItemKind::Literal(b)) => a.partial_cmp(b),
            (ItemKind::Lang(na, a), ItemKind::Lang(nb, b)) => match na.cmp(nb) {
                Ordering::Equal => PartialOrd::partial_cmp(a.as_ref(), b.as_ref()),
                o => Some(o),
            },
            (ItemKind::Register(na, a), ItemKind::Register(nb, b)) => match na.cmp(nb) {
                Ordering::Equal => PartialOrd::partial_cmp(a.as_ref(), b.as_ref()),
                o => Some(o),
            },
            (ItemKind::Push, ItemKind::Push) => Some(Ordering::Equal),
            (ItemKind::Line, ItemKind::Line) => Some(Ordering::Equal),
            (ItemKind::Space, ItemKind::Space) => Some(Ordering::Equal),
            (ItemKind::Indentation(na), ItemKind::Indentation(nb)) => na.partial_cmp(nb),
            (ItemKind::OpenQuote(ba), ItemKind::OpenQuote(bb)) => ba.partial_cmp(bb),
            (ItemKind::CloseQuote, ItemKind::CloseQuote) => Some(Ordering::Equal),
            (ItemKind::OpenEval, ItemKind::OpenEval) => Some(Ordering::Equal),
            (ItemKind::CloseEval, ItemKind::CloseEval) => Some(Ordering::Equal),
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
        // NB: This is here because it can't be derived due to the generic
        // parameter `L` not implementing `Ord`.
        match (&self.kind, &other.kind) {
            (ItemKind::Literal(a), ItemKind::Literal(b)) => a.cmp(b),
            (ItemKind::Lang(na, a), ItemKind::Lang(nb, b)) => match na.cmp(nb) {
                Ordering::Equal => Ord::cmp(a.as_ref(), b.as_ref()),
                o => o,
            },
            (ItemKind::Register(na, a), ItemKind::Register(nb, b)) => match na.cmp(nb) {
                Ordering::Equal => Ord::cmp(a.as_ref(), b.as_ref()),
                o => o,
            },
            (ItemKind::Push, ItemKind::Push) => Ordering::Equal,
            (ItemKind::Line, ItemKind::Line) => Ordering::Equal,
            (ItemKind::Space, ItemKind::Space) => Ordering::Equal,
            (ItemKind::Indentation(na), ItemKind::Indentation(nb)) => na.cmp(nb),
            (ItemKind::OpenQuote(ba), ItemKind::OpenQuote(bb)) => ba.cmp(bb),
            (ItemKind::CloseQuote, ItemKind::CloseQuote) => Ordering::Equal,
            (ItemKind::OpenEval, ItemKind::OpenEval) => Ordering::Equal,
            (ItemKind::CloseEval, ItemKind::CloseEval) => Ordering::Equal,
            (ItemKind::Literal(_), _) => Ordering::Less,
            (_, ItemKind::Literal(_)) => Ordering::Greater,
            (ItemKind::Lang(_, _), _) => Ordering::Less,
            (_, ItemKind::Lang(_, _)) => Ordering::Greater,
            (ItemKind::Register(_, _), _) => Ordering::Less,
            (_, ItemKind::Register(_, _)) => Ordering::Greater,
            (ItemKind::Push, _) => Ordering::Less,
            (_, ItemKind::Push) => Ordering::Greater,
            (ItemKind::Line, _) => Ordering::Less,
            (_, ItemKind::Line) => Ordering::Greater,
            (ItemKind::Space, _) => Ordering::Less,
            (_, ItemKind::Space) => Ordering::Greater,
            (ItemKind::Indentation(_), _) => Ordering::Less,
            (_, ItemKind::Indentation(_)) => Ordering::Greater,
            (ItemKind::OpenQuote(_), _) => Ordering::Less,
            (_, ItemKind::OpenQuote(_)) => Ordering::Greater,
            (ItemKind::CloseQuote, _) => Ordering::Less,
            (_, ItemKind::CloseQuote) => Ordering::Greater,
            (ItemKind::OpenEval, _) => Ordering::Less,
            (_, ItemKind::OpenEval) => Ordering::Greater,
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

        match &self.kind {
            ItemKind::Literal(item_str) => {
                item_str.hash(state);
            }
            ItemKind::Lang(n, item) => {
                n.hash(state);
                item.hash(state);
            }
            ItemKind::Register(n, item) => {
                n.hash(state);
                item.hash(state);
            }
            ItemKind::Push => {}
            ItemKind::Line => {}
            ItemKind::Space => {}
            ItemKind::Indentation(n) => {
                n.hash(state);
            }
            ItemKind::OpenQuote(n) => {
                n.hash(state);
            }
            ItemKind::CloseQuote => {}
            ItemKind::OpenEval => {}
            ItemKind::CloseEval => {}
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
/// let foo = Item::literal(ItemStr::static_("foo"));
/// let bar = Item::literal("bar".into());
///
/// let result: Tokens = quote!($foo $bar baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
///
/// assert_eq!{
///     vec![
///         Item::literal(ItemStr::static_("foo")),
///         Item::space(),
///         Item::literal("bar".into()),
///         Item::space(),
///         Item::literal(ItemStr::static_("baz")),
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
