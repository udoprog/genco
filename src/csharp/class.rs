//! Data structure for classes.

use super::constructor::Constructor;
use super::field::Field;
use super::method::Method;
use super::modifier::Modifier;
use cons::Cons;
use csharp::Csharp;
use element::Element;
use into_tokens::IntoTokens;
use tokens::Tokens;

/// Model for Csharp Classs.
#[derive(Debug, Clone)]
pub struct Class<'el> {
    /// Class modifiers.
    pub modifiers: Vec<Modifier>,
    /// Declared methods.
    pub fields: Vec<Field<'el>>,
    /// Declared methods.
    pub constructors: Vec<Constructor<'el>>,
    /// Declared methods.
    pub methods: Vec<Method<'el>>,
    /// Extra body (at the end of the class).
    pub body: Tokens<'el, Csharp<'el>>,
    /// What this class extends.
    pub extends: Option<Csharp<'el>>,
    /// What this class implements.
    pub implements: Vec<Csharp<'el>>,
    /// Generic parameters.
    pub parameters: Tokens<'el, Csharp<'el>>,
    /// Attributes for the constructor.
    attributes: Tokens<'el, Csharp<'el>>,
    /// Name of class.
    name: Cons<'el>,
}

impl<'el> Class<'el> {
    /// Build a new empty interface.
    pub fn new<N>(name: N) -> Class<'el>
    where
        N: Into<Cons<'el>>,
    {
        Class {
            modifiers: vec![Modifier::Public],
            fields: vec![],
            methods: vec![],
            body: Tokens::new(),
            constructors: vec![],
            extends: None,
            implements: vec![],
            parameters: Tokens::new(),
            attributes: Tokens::new(),
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

    /// Name of class.
    pub fn name(&self) -> Cons<'el> {
        self.name.clone()
    }
}

into_tokens_impl_from!(Class<'el>, Csharp<'el>);

impl<'el> IntoTokens<'el, Csharp<'el>> for Class<'el> {
    fn into_tokens(self) -> Tokens<'el, Csharp<'el>> {
        let mut sig: Tokens<Csharp> = Tokens::new();

        sig.extend(self.modifiers.into_tokens());

        sig.append("class");

        sig.append({
            let mut n = Tokens::new();
            n.append(self.name.clone());

            if !self.parameters.is_empty() {
                n.append("<");
                n.append(self.parameters.join(", "));
                n.append(">");
            }

            n
        });

        let mut extends = Tokens::new();

        extends.extend(self.extends.into_iter().map(Element::from));
        extends.extend(self.implements.into_iter().map(Element::from));

        if !extends.is_empty() {
            sig.append(":");
            sig.append(extends.join(", "));
        }

        let mut s = Tokens::new();

        if !self.attributes.is_empty() {
            s.push(self.attributes);
        }

        s.push(toks![sig.join_spacing(), " {"]);

        s.nested({
            let mut body = Tokens::new();

            if !self.fields.is_empty() {
                let mut fields = Tokens::new();

                for field in self.fields {
                    if field.block.is_some() {
                        fields.push(field);
                    } else {
                        fields.push(toks![field, ";"]);
                    }
                }

                body.push(fields);
            }

            if !self.constructors.is_empty() {
                for constructor in self.constructors {
                    body.push((self.name.clone(), constructor));
                }
            }

            if !self.methods.is_empty() {
                for method in self.methods {
                    body.push(method);
                }
            }

            body.extend(self.body);
            body.join_line_spacing()
        });

        s.push("}");

        s
    }
}

#[cfg(test)]
mod tests {
    use super::Class;
    use csharp::{local, Csharp};
    use tokens::Tokens;

    #[test]
    fn test_class() {
        let mut c = Class::new("Foo");
        c.parameters.append("T");
        c.implements = vec![local("Super").into()];

        let t: Tokens<Csharp> = c.into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public class Foo<T> : Super {\n}"), out);
    }
}
