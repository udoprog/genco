//! Data structure for constructors

use super::modifier::Modifier;
use cons::Cons;
use csharp::Csharp;
use into_tokens::IntoTokens;
use tokens::Tokens;

/// Model for C# Arguments to functions.
#[derive(Debug, Clone)]
pub struct Argument<'el> {
    /// Modifiers for argument.
    pub modifiers: Vec<Modifier>,
    /// Attributes to argument.
    attributes: Tokens<'el, Csharp<'el>>,
    /// Type of argument.
    ty: Csharp<'el>,
    /// Name of argument.
    name: Cons<'el>,
}

impl<'el> Argument<'el> {
    /// Build a new empty argument.
    pub fn new<T, N>(ty: T, name: N) -> Argument<'el>
    where
        T: Into<Csharp<'el>>,
        N: Into<Cons<'el>>,
    {
        Argument {
            attributes: Tokens::new(),
            modifiers: vec![],
            ty: ty.into(),
            name: name.into(),
        }
    }

    /// Push an attribute.
    pub fn attribute<T>(&mut self, attribute: T)
    where
        T: IntoTokens<'el, Csharp<'el>>,
    {
        self.attributes.push(attribute.into_tokens());
    }

    /// Get the variable of the argument.
    pub fn var(&self) -> Cons<'el> {
        self.name.clone()
    }

    /// The type of the argument.
    pub fn ty(&self) -> Csharp<'el> {
        self.ty.clone()
    }
}

into_tokens_impl_from!(Argument<'el>, Csharp<'el>);

impl<'el> IntoTokens<'el, Csharp<'el>> for Argument<'el> {
    fn into_tokens(self) -> Tokens<'el, Csharp<'el>> {
        let mut s = Tokens::new();

        s.extend(self.attributes.into_iter());
        s.extend(self.modifiers.into_tokens());
        s.append(self.ty);
        s.append(self.name);

        s.join_spacing()
    }
}
