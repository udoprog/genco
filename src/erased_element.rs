use crate::Cons;

/// A type-erased variant of element, useful for constructing elements which are
/// not associated with any specific customization.
#[derive(Clone)]
pub enum ErasedElement<'el> {
    /// A borrowed quoted string.
    Quoted(Cons<'el>),
}
