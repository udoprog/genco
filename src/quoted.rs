//! Trait to convert to quoted.

use super::cons::Cons;
use super::element::Element;
use std::rc::Rc;

/// Trait to convert types to quoted elements.
pub trait Quoted<'el> {
    /// Convert type to quoted element.
    fn quoted<C>(self) -> Element<'el, C>;
}

impl<'el> Quoted<'el> for String {
    fn quoted<C>(self) -> Element<'el, C> {
        Element::Quoted(Cons::Rc(Rc::new(self)))
    }
}

impl<'el> Quoted<'el> for &'el str {
    fn quoted<C>(self) -> Element<'el, C> {
        Element::Quoted(Cons::Borrowed(self))
    }
}

impl<'el> Quoted<'el> for Rc<String> {
    fn quoted<C>(self) -> Element<'el, C> {
        Element::Quoted(Cons::Rc(self))
    }
}

impl<'el> Quoted<'el> for Cons<'el> {
    fn quoted<C>(self) -> Element<'el, C> {
        Element::Quoted(self)
    }
}
