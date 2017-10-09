//! Trait to convert to quoted.

use super::element::Element;
use super::cons::Cons;
use std::rc::Rc;

/// Trait to convert types to quoted elements.
pub trait Quoted<'element> {
    /// Convert type to quoted element.
    fn quoted<C>(self) -> Element<'element, C>;
}

impl<'element> Quoted<'element> for String {
    fn quoted<C>(self) -> Element<'element, C> {
        Element::Quoted(Cons::Owned(self))
    }
}

impl<'element> Quoted<'element> for &'element str {
    fn quoted<C>(self) -> Element<'element, C> {
        Element::Quoted(Cons::Borrowed(self))
    }
}

impl<'element> Quoted<'element> for Rc<String> {
    fn quoted<C>(self) -> Element<'element, C> {
        Element::Quoted(Cons::Rc(self))
    }
}
