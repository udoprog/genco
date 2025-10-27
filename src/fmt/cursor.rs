use crate::fmt;
use crate::tokens::{Item, Kind};

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
        matches!(item.kind, Kind::Literal(..))
    }

    #[inline]
    fn parse(item: &Item) -> fmt::Result<&Self::Output> {
        match &item.kind {
            Kind::Literal(s) => Ok(s),
            _ => Err(core::fmt::Error),
        }
    }
}

/// Peek for an eval marker.
pub(super) struct CloseEval(());

impl Parse for CloseEval {
    type Output = ();

    #[inline]
    fn peek(item: &Item) -> bool {
        matches!(item.kind, Kind::CloseEval)
    }

    #[inline]
    fn parse(item: &Item) -> fmt::Result<&Self::Output> {
        match &item.kind {
            Kind::CloseEval => Ok(&()),
            _ => Err(core::fmt::Error),
        }
    }
}

/// Parser helper.
pub(super) struct Cursor<'a, T> {
    lang: &'a [T],
    items: &'a [Item],
}

impl<'a, T> Cursor<'a, T> {
    /// Construct a new cursor.
    pub(super) fn new(lang: &'a [T], items: &'a [Item]) -> Self {
        Self { lang, items }
    }

    /// Get a language item by index.
    pub(super) fn lang(&self, index: usize) -> fmt::Result<&'a T> {
        self.lang.get(index).ok_or(core::fmt::Error)
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
        if let Some(item) = self.items.first() {
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
        let item = self.next().ok_or(core::fmt::Error)?;
        P::parse(item)
    }
}
