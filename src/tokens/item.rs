//! A single element

use core::cmp::Ordering;
use core::fmt;
use core::hash;
use core::mem;

use alloc::boxed::Box;

use crate::lang::Lang;
use crate::tokens::{FormatInto, ItemStr, Tokens};

pub(crate) enum Kind<L>
where
    L: Lang,
{
    /// A literal item.
    /// Is added as a raw string to the stream of tokens.
    Literal(ItemStr),
    /// A language-specific item.
    Lang(Box<L::Item>),
    /// A language-specific item that is not rendered.
    Register(Box<L::Item>),
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
    pub(crate) kind: Kind<L>,
}

impl<L> Item<L>
where
    L: Lang,
{
    /// Construct a new item based on a kind.
    #[inline]
    pub(crate) const fn new(kind: Kind<L>) -> Self {
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
        Self::new(Kind::Literal(ItemStr::static_(literal)))
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
        Self::new(Kind::Push)
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
        Self::new(Kind::Line)
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
        Self::new(Kind::Space)
    }

    /// Construct a new indentation item.
    #[inline]
    pub(crate) const fn indentation(n: i16) -> Self {
        Self::new(Kind::Indentation(n))
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
        Self::new(Kind::OpenQuote(is_interpolated))
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
        Self::new(Kind::CloseQuote)
    }

    /// Construct a new eval open.
    #[inline]
    pub(crate) const fn open_eval() -> Self {
        Self::new(Kind::OpenEval)
    }

    /// Construct a new eval close.
    #[inline]
    pub(crate) const fn close_eval() -> Self {
        Self::new(Kind::CloseEval)
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
        Self::new(Kind::Literal(lit))
    }

    /// Construct a new language-specific item.
    #[inline]
    pub(crate) const fn lang(item: Box<L::Item>) -> Self {
        Self::new(Kind::Lang(item))
    }

    /// Construct a new language-specific register item.
    #[inline]
    pub(crate) const fn register(item: Box<L::Item>) -> Self {
        Self::new(Kind::Register(item))
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
            Kind::Literal(lit) => f.debug_tuple("Literal").field(lit).finish(),
            Kind::Lang(item) => f.debug_tuple("Lang").field(item).finish(),
            Kind::Register(item) => f.debug_tuple("Register").field(item).finish(),
            Kind::Push => write!(f, "Push"),
            Kind::Line => write!(f, "Line"),
            Kind::Space => write!(f, "Space"),
            Kind::Indentation(n) => f.debug_tuple("Indentation").field(n).finish(),
            Kind::OpenQuote(b) => f.debug_tuple("OpenQuote").field(b).finish(),
            Kind::CloseQuote => write!(f, "CloseQuote"),
            Kind::OpenEval => write!(f, "OpenEval"),
            Kind::CloseEval => write!(f, "CloseEval"),
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
            Kind::Literal(lit) => Kind::Literal(lit.clone()),
            Kind::Lang(item) => Kind::Lang(item.clone()),
            Kind::Register(item) => Kind::Register(item.clone()),
            Kind::Push => Kind::Push,
            Kind::Line => Kind::Line,
            Kind::Space => Kind::Space,
            Kind::Indentation(n) => Kind::Indentation(*n),
            Kind::OpenQuote(b) => Kind::OpenQuote(*b),
            Kind::CloseQuote => Kind::CloseQuote,
            Kind::OpenEval => Kind::OpenEval,
            Kind::CloseEval => Kind::CloseEval,
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
            (Kind::Literal(a), Kind::Literal(b)) => a == b,
            (Kind::Lang(a), Kind::Lang(b)) => **a == **b,
            (Kind::Register(a), Kind::Register(b)) => **a == **b,
            (Kind::Push, Kind::Push) => true,
            (Kind::Line, Kind::Line) => true,
            (Kind::Space, Kind::Space) => true,
            (Kind::Indentation(na), Kind::Indentation(nb)) => na == nb,
            (Kind::OpenQuote(ba), Kind::OpenQuote(bb)) => ba == bb,
            (Kind::CloseQuote, Kind::CloseQuote) => true,
            (Kind::OpenEval, Kind::OpenEval) => true,
            (Kind::CloseEval, Kind::CloseEval) => true,
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
            (Kind::Literal(a), Kind::Literal(b)) => a.partial_cmp(b),
            (Kind::Lang(a), Kind::Lang(b)) => a.as_ref().partial_cmp(b.as_ref()),
            (Kind::Register(a), Kind::Register(b)) => a.as_ref().partial_cmp(b.as_ref()),
            (Kind::Push, Kind::Push) => Some(Ordering::Equal),
            (Kind::Line, Kind::Line) => Some(Ordering::Equal),
            (Kind::Space, Kind::Space) => Some(Ordering::Equal),
            (Kind::Indentation(na), Kind::Indentation(nb)) => na.partial_cmp(nb),
            (Kind::OpenQuote(ba), Kind::OpenQuote(bb)) => ba.partial_cmp(bb),
            (Kind::CloseQuote, Kind::CloseQuote) => Some(Ordering::Equal),
            (Kind::OpenEval, Kind::OpenEval) => Some(Ordering::Equal),
            (Kind::CloseEval, Kind::CloseEval) => Some(Ordering::Equal),
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
            (Kind::Literal(a), Kind::Literal(b)) => a.cmp(b),
            (Kind::Lang(a), Kind::Lang(b)) => a.as_ref().cmp(b),
            (Kind::Register(a), Kind::Register(b)) => a.as_ref().cmp(b.as_ref()),
            (Kind::Push, Kind::Push) => Ordering::Equal,
            (Kind::Line, Kind::Line) => Ordering::Equal,
            (Kind::Space, Kind::Space) => Ordering::Equal,
            (Kind::Indentation(na), Kind::Indentation(nb)) => na.cmp(nb),
            (Kind::OpenQuote(ba), Kind::OpenQuote(bb)) => ba.cmp(bb),
            (Kind::CloseQuote, Kind::CloseQuote) => Ordering::Equal,
            (Kind::OpenEval, Kind::OpenEval) => Ordering::Equal,
            (Kind::CloseEval, Kind::CloseEval) => Ordering::Equal,
            (Kind::Literal(_), _) => Ordering::Less,
            (_, Kind::Literal(_)) => Ordering::Greater,
            (Kind::Lang(_), _) => Ordering::Less,
            (_, Kind::Lang(_)) => Ordering::Greater,
            (Kind::Register(_), _) => Ordering::Less,
            (_, Kind::Register(_)) => Ordering::Greater,
            (Kind::Push, _) => Ordering::Less,
            (_, Kind::Push) => Ordering::Greater,
            (Kind::Line, _) => Ordering::Less,
            (_, Kind::Line) => Ordering::Greater,
            (Kind::Space, _) => Ordering::Less,
            (_, Kind::Space) => Ordering::Greater,
            (Kind::Indentation(_), _) => Ordering::Less,
            (_, Kind::Indentation(_)) => Ordering::Greater,
            (Kind::OpenQuote(_), _) => Ordering::Less,
            (_, Kind::OpenQuote(_)) => Ordering::Greater,
            (Kind::CloseQuote, _) => Ordering::Less,
            (_, Kind::CloseQuote) => Ordering::Greater,
            (Kind::OpenEval, _) => Ordering::Less,
            (_, Kind::OpenEval) => Ordering::Greater,
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
            Kind::Literal(item_str) => {
                item_str.hash(state);
            }
            Kind::Lang(item) => {
                item.hash(state);
            }
            Kind::Register(item) => {
                item.hash(state);
            }
            Kind::Push => {}
            Kind::Line => {}
            Kind::Space => {}
            Kind::Indentation(n) => {
                n.hash(state);
            }
            Kind::OpenQuote(n) => {
                n.hash(state);
            }
            Kind::CloseQuote => {}
            Kind::OpenEval => {}
            Kind::CloseEval => {}
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
