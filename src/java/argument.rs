//! Data structure for constructors

use tokens::Tokens;
use java::Java;
use super::modifier::Modifier;
use cons::Cons;
use into_tokens::IntoTokens;

/// Model for Java Arguments to functions.
#[derive(Debug, Clone)]
pub struct Argument<'el> {
    /// Modifiers for argument.
    pub modifiers: Vec<Modifier>,
    /// Annotations to argument.
    annotations: Tokens<'el, Java<'el>>,
    /// Type of argument.
    ty: Java<'el>,
    /// Name of argument.
    name: Cons<'el>,
}

impl<'el> Argument<'el> {
    /// Build a new empty argument.
    pub fn new<T, N>(ty: T, name: N) -> Argument<'el>
    where
        T: Into<Java<'el>>,
        N: Into<Cons<'el>>,
    {
        Argument {
            annotations: Tokens::new(),
            modifiers: vec![Modifier::Final],
            ty: ty.into(),
            name: name.into(),
        }
    }

    /// Push an annotation.
    pub fn annotation<A>(&mut self, annotation: A)
    where
        A: IntoTokens<'el, Java<'el>>,
    {
        self.annotations.push(annotation.into_tokens());
    }

    /// Get the variable of the argument.
    pub fn var(&self) -> Cons<'el> {
        self.name.clone()
    }
}

into_tokens_impl_from!(Argument<'el>, Java<'el>);

impl<'el> IntoTokens<'el, Java<'el>> for Argument<'el> {
    fn into_tokens(self) -> Tokens<'el, Java<'el>> {
        let mut s = Tokens::new();

        let modifiers: Tokens<Java> = self.modifiers.into();

        s.extend(self.annotations.join_spacing());
        s.extend(modifiers);
        s.append(self.ty);
        s.append(self.name);

        s.join_spacing()
    }
}
