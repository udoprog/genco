use crate::fmt;
use crate::tokens::Item;

/// Trait for peeking items.
pub(super) trait Parse {
    type Output: ?Sized;

    /// Parse the given item into its output.
    fn parse(item: &Item) -> fmt::Result<&Self::Output>;

    /// Test if the peek matches the given item.
    fn peek(item: &Item) -> bool;
}

/// Peek for a literal.
pub(super) struct Literal(());

impl Parse for Literal {
    type Output = str;

    #[inline]
    fn peek(item: &Item) -> bool {
        match item {
            Item::Literal(..) => true,
            _ => false,
        }
    }

    #[inline]
    fn parse(item: &Item) -> fmt::Result<&Self::Output> {
        match item {
            Item::Literal(s) => Ok(s),
            _ => Err(std::fmt::Error),
        }
    }
}

/// Peek for an eval marker.
pub(super) struct CloseEval(());

impl Parse for CloseEval {
    type Output = ();

    #[inline]
    fn peek(item: &Item) -> bool {
        match item {
            Item::CloseEval => true,
            _ => false,
        }
    }

    #[inline]
    fn parse(item: &Item) -> fmt::Result<&Self::Output> {
        match item {
            Item::CloseEval => Ok(&()),
            _ => Err(std::fmt::Error),
        }
    }
}

/// Parser helper.
pub(super) struct Cursor<'a> {
    items: &'a [Item],
}

impl<'a> Cursor<'a> {
    pub(super) fn new(items: &'a [Item]) -> Self {
        Self { items }
    }

    /// Get the next item.
    pub(super) fn next(&mut self) -> Option<&Item> {
        let (first, rest) = self.items.split_first()?;
        self.items = rest;
        Some(first)
    }

    #[inline]
    pub(super) fn peek<P>(&self) -> bool
    where
        P: Parse,
    {
        if let Some(item) = self.items.get(0) {
            P::peek(item)
        } else {
            false
        }
    }

    #[inline]
    pub(super) fn peek1<P>(&self) -> bool
    where
        P: Parse,
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
        P: Parse,
    {
        let item = self.next().ok_or_else(|| std::fmt::Error)?;
        P::parse(item)
    }
}
