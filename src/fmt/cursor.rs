use crate::fmt;
use crate::lang::Lang;
use crate::tokens::Item;

/// Trait for peeking items.
pub(super) trait Parse<L>
where
    L: Lang,
{
    type Output: ?Sized;

    /// Parse the given item into its output.
    fn parse(item: &Item<L>) -> fmt::Result<&Self::Output>;

    /// Test if the peek matches the given item.
    fn peek(item: &Item<L>) -> bool;
}

/// Peek for a literal.
pub(super) struct Literal(());

impl<L> Parse<L> for Literal
where
    L: Lang,
{
    type Output = str;

    #[inline]
    fn peek(item: &Item<L>) -> bool {
        matches!(item, Item::Literal(..))
    }

    #[inline]
    fn parse(item: &Item<L>) -> fmt::Result<&Self::Output> {
        match item {
            Item::Literal(s) => Ok(s),
            _ => Err(core::fmt::Error),
        }
    }
}

/// Peek for an eval marker.
pub(super) struct CloseEval(());

impl<L> Parse<L> for CloseEval
where
    L: Lang,
{
    type Output = ();

    #[inline]
    fn peek(item: &Item<L>) -> bool {
        matches!(item, Item::CloseEval)
    }

    #[inline]
    fn parse(item: &Item<L>) -> fmt::Result<&Self::Output> {
        match item {
            Item::CloseEval => Ok(&()),
            _ => Err(core::fmt::Error),
        }
    }
}

/// Parser helper.
pub(super) struct Cursor<'a, L>
where
    L: Lang,
{
    items: &'a [Item<L>],
}

impl<'a, L> Cursor<'a, L>
where
    L: Lang,
{
    pub(super) fn new(items: &'a [Item<L>]) -> Self {
        Self { items }
    }

    /// Get the next item.
    pub(super) fn next(&mut self) -> Option<&Item<L>> {
        let (first, rest) = self.items.split_first()?;
        self.items = rest;
        Some(first)
    }

    #[inline]
    pub(super) fn peek<P>(&self) -> bool
    where
        P: Parse<L>,
    {
        if let Some(item) = self.items.first() {
            P::peek(item)
        } else {
            false
        }
    }

    #[inline]
    pub(super) fn peek1<P>(&self) -> bool
    where
        P: Parse<L>,
    {
        if let Some(item) = self.items.get(1) {
            P::peek(item)
        } else {
            false
        }
    }

    #[inline]
    pub(super) fn parse<P>(&mut self) -> fmt::Result<&P::Output>
    where
        P: Parse<L>,
    {
        let item = self.next().ok_or(core::fmt::Error)?;
        P::parse(item)
    }
}
