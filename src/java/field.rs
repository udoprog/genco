//! Data structure for fields

use tokens::Tokens;
use java::Java;
use cons::Cons;
use con::Con;
use super::modifier::Modifier;
use element::Element;

/// Model for Java Fields.
#[derive(Debug, Clone)]
pub struct Field<'el> {
    /// Annotations of field.
    pub annotations: Tokens<'el, Java<'el>>,
    /// Modifiers of field.
    pub modifiers: Vec<Modifier>,
    /// Type of field.
    ty: Java<'el>,
    /// Name of field.
    name: Cons<'el>,
    /// Initializer of field.
    initializer: Option<Tokens<'el, Java<'el>>>,
}

impl<'el> Field<'el> {
    /// Create a new field.
    pub fn new<T, N>(ty: T, name: N) -> Field<'el>
    where
        T: Into<Java<'el>>,
        N: Into<Cons<'el>>,
    {
        use self::Modifier::*;

        Field {
            annotations: Tokens::new(),
            modifiers: vec![Private, Final],
            ty: ty.into(),
            name: name.into(),
            initializer: None,
        }
    }

    /// Push an annotation.
    pub fn annotation<A>(&mut self, annotation: A)
    where
        A: Into<Tokens<'el, Java<'el>>>,
    {
        self.annotations.push(annotation.into());
    }

    /// Set initializer for field.
    pub fn initializer<I>(&mut self, initializer: I)
    where
        I: Into<Tokens<'el, Java<'el>>>,
    {
        self.initializer = Some(initializer.into());
    }

    /// The variable of the field.
    pub fn var(&self) -> Cons<'el> {
        self.name.clone()
    }

    /// The type of the field.
    pub fn ty(&self) -> Java<'el> {
        self.ty.clone()
    }
}

impl<'el> From<Field<'el>> for Tokens<'el, Java<'el>> {
    fn from(f: Field<'el>) -> Self {
        let mut tokens = Tokens::new();

        if !f.annotations.is_empty() {
            tokens.push(f.annotations);
            tokens.append(Element::PushLine);
        }

        if !f.modifiers.is_empty() {
            tokens.append(f.modifiers);
            tokens.append(" ");
        }

        tokens.append(f.ty);
        tokens.append(" ");
        tokens.append(f.name);

        if let Some(initializer) = f.initializer {
            tokens.append(" = ");
            tokens.append(initializer);
        }

        tokens
    }
}

impl<'el> From<Field<'el>> for Element<'el, Java<'el>> {
    fn from(f: Field<'el>) -> Self {
        Element::Append(Con::Owned(f.into()))
    }
}
