//! Trait to convert to quoted.

use crate::{cons::Cons, ErasedElement};
use std::rc::Rc;

/// Trait to convert types to quoted elements.
pub trait Quoted<'el> {
    /// Convert type to quoted element.
    fn quoted(self) -> ErasedElement<'el>;
}

impl<'el> Quoted<'el> for String {
    fn quoted(self) -> ErasedElement<'el> {
        ErasedElement::Quoted(Cons::Rc(Rc::new(self)))
    }
}

impl<'el> Quoted<'el> for &'el str {
    fn quoted(self) -> ErasedElement<'el> {
        ErasedElement::Quoted(Cons::Borrowed(self))
    }
}

impl<'el> Quoted<'el> for Rc<String> {
    fn quoted(self) -> ErasedElement<'el> {
        ErasedElement::Quoted(Cons::Rc(self))
    }
}

impl<'el> Quoted<'el> for Cons<'el> {
    fn quoted(self) -> ErasedElement<'el> {
        ErasedElement::Quoted(self)
    }
}
