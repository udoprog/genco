//! Data structure for fields

use {Cons, Csharp, Element, IntoTokens, Tokens};
use con_::Con;
use csharp::{BlockComment, Modifier};

/// Model for Csharp Fields.
#[derive(Debug, Clone)]
pub struct Field<'el> {
    /// Attributes of field.
    pub attributes: Tokens<'el, Csharp<'el>>,
    /// Modifiers of field.
    pub modifiers: Vec<Modifier>,
    /// Comments associated with this field.
    pub comments: Vec<Cons<'el>>,
    /// Block of field.
    pub block: Option<Tokens<'el, Csharp<'el>>>,
    /// Type of field.
    ty: Csharp<'el>,
    /// Name of field.
    name: Cons<'el>,
}

impl<'el> Field<'el> {
    /// Create a new field.
    pub fn new<T, N>(ty: T, name: N) -> Field<'el>
    where
        T: Into<Csharp<'el>>,
        N: Into<Cons<'el>>,
    {
        use self::Modifier::*;

        Field {
            attributes: Tokens::new(),
            modifiers: vec![Private],
            comments: vec![],
            block: None,
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

    /// Set block for field.
    pub fn block<I>(&mut self, block: I)
    where
        I: IntoTokens<'el, Csharp<'el>>,
    {
        self.block = Some(block.into_tokens());
    }

    /// The variable of the field.
    pub fn var(&self) -> Cons<'el> {
        self.name.clone()
    }

    /// The type of the field.
    pub fn ty(&self) -> Csharp<'el> {
        self.ty.clone()
    }
}

into_tokens_impl_from!(Field<'el>, Csharp<'el>);

impl<'el> IntoTokens<'el, Csharp<'el>> for Field<'el> {
    fn into_tokens(self) -> Tokens<'el, Csharp<'el>> {
        let mut tokens = Tokens::new();

        tokens.push_unless_empty(BlockComment(self.comments));

        if !self.attributes.is_empty() {
            tokens.push(self.attributes);
            tokens.append(Element::PushSpacing);
        }

        tokens.append({
            let mut sig = Tokens::new();

            sig.extend(self.modifiers.into_tokens());

            sig.append(self.ty);
            sig.append(self.name);

            if let Some(block) = self.block {
                sig.append({
                    let mut b = Tokens::new();
                    b.append("{");
                    b.nested(block);
                    b.push("}");
                    b
                });
            }

            sig.join_spacing()
        });

        tokens
    }
}

impl<'el> From<Field<'el>> for Element<'el, Csharp<'el>> {
    fn from(f: Field<'el>) -> Self {
        Element::Append(Con::Owned(f.into_tokens()))
    }
}

#[cfg(test)]
mod tests {
    use csharp::{Field, INT32};
    use tokens::Tokens;

    fn field() -> Field<'static> {
        Field::new(INT32, "foo")
    }

    #[test]
    fn test_with_comments() {
        let mut c = field();
        c.comments.push("Hello World".into());
        let t: Tokens<_> = c.into();
        assert_eq!(
            Ok(String::from("/// Hello World\nprivate Int32 foo")),
            t.to_string()
        );
    }

    #[test]
    fn test_no_comments() {
        let t = Tokens::from(field());
        assert_eq!(Ok(String::from("private Int32 foo")), t.to_string());
    }
}
