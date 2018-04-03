//! Data structure for fields

use con_::Con;
use java::{BlockComment, Modifier};
use {Cons, Element, IntoTokens, Java, Tokens};

/// Model for Java Fields.
#[derive(Debug, Clone)]
pub struct Field<'el> {
    /// Annotations of field.
    pub annotations: Tokens<'el, Java<'el>>,
    /// Modifiers of field.
    pub modifiers: Vec<Modifier>,
    /// Comments associated with this field.
    pub comments: Vec<Cons<'el>>,
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
            comments: vec![],
            ty: ty.into(),
            name: name.into(),
            initializer: None,
        }
    }

    /// Push an annotation.
    pub fn annotation<A>(&mut self, annotation: A)
    where
        A: IntoTokens<'el, Java<'el>>,
    {
        self.annotations.push(annotation.into_tokens());
    }

    /// Set initializer for field.
    pub fn initializer<I>(&mut self, initializer: I)
    where
        I: IntoTokens<'el, Java<'el>>,
    {
        self.initializer = Some(initializer.into_tokens());
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

into_tokens_impl_from!(Field<'el>, Java<'el>);

impl<'el> IntoTokens<'el, Java<'el>> for Field<'el> {
    fn into_tokens(self) -> Tokens<'el, Java<'el>> {
        let mut tokens = Tokens::new();

        tokens.push_unless_empty(BlockComment(self.comments));

        if !self.annotations.is_empty() {
            tokens.push(self.annotations);
            tokens.append(Element::PushSpacing);
        }

        tokens.append({
            let mut sig = Tokens::new();

            sig.extend(self.modifiers.into_tokens());

            sig.append(self.ty);
            sig.append(self.name);

            if let Some(initializer) = self.initializer {
                sig.append("=");
                sig.append(initializer);
            }

            sig.join_spacing()
        });

        tokens
    }
}

impl<'el> From<Field<'el>> for Element<'el, Java<'el>> {
    fn from(f: Field<'el>) -> Self {
        Element::Append(Con::Owned(f.into_tokens()))
    }
}

#[cfg(test)]
mod tests {
    use java::{Field, INTEGER};
    use tokens::Tokens;

    fn field() -> Field<'static> {
        Field::new(INTEGER, "foo")
    }

    #[test]
    fn test_with_comments() {
        let mut c = field();
        c.comments.push("Hello World".into());
        let t: Tokens<_> = c.into();
        assert_eq!(
            Ok(String::from(
                "/**\n * Hello World\n */\nprivate final int foo",
            )),
            t.to_string()
        );
    }

    #[test]
    fn test_no_comments() {
        let t = Tokens::from(field());
        assert_eq!(Ok(String::from("private final int foo")), t.to_string());
    }
}
