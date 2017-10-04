//! Trait to convert to quoted.

use super::element::Element;
use std::borrow::Cow;

/// Trait to convert types to quoted elements.
pub trait Quoted<'element> {
    /// Convert type to quoted element.
    fn quoted<C>(self) -> Element<'element, C>;
}

impl<'element> Quoted<'element> for String {
    fn quoted<C>(self) -> Element<'element, C> {
        Element::Quoted(Cow::Owned(self))
    }
}

impl<'element> Quoted<'element> for &'element str {
    fn quoted<C>(self) -> Element<'element, C> {
        Element::Quoted(Cow::Borrowed(self))
    }
}
