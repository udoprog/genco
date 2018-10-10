//! Data structure for interfaces.

use csharp::{Method, Modifier};
use {Cons, Csharp, Element, IntoTokens, Tokens};

/// Model for Csharp Interfaces.
#[derive(Debug, Clone)]
pub struct Interface<'el> {
    /// Interface modifiers.
    pub modifiers: Vec<Modifier>,
    /// Declared methods.
    pub methods: Vec<Method<'el>>,
    /// Extra body (added to end of interface).
    pub body: Tokens<'el, Csharp<'el>>,
    /// What this interface extends.
    pub extends: Vec<Csharp<'el>>,
    /// Generic parameters.
    pub parameters: Tokens<'el, Csharp<'el>>,
    /// Attributes for the constructor.
    attributes: Tokens<'el, Csharp<'el>>,
    /// Name of interface.
    name: Cons<'el>,
}

impl<'el> Interface<'el> {
    /// Build a new empty interface.
    pub fn new<N>(name: N) -> Interface<'el>
    where
        N: Into<Cons<'el>>,
    {
        Interface {
            modifiers: vec![Modifier::Public],
            methods: vec![],
            body: Tokens::new(),
            extends: vec![],
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

    /// Name of interface.
    pub fn name(&self) -> Cons<'el> {
        self.name.clone()
    }
}

into_tokens_impl_from!(Interface<'el>, Csharp<'el>);

impl<'el> IntoTokens<'el, Csharp<'el>> for Interface<'el> {
    fn into_tokens(self) -> Tokens<'el, Csharp<'el>> {
        let mut sig: Tokens<Csharp> = Tokens::new();

        sig.extend(self.modifiers.into_tokens());

        sig.append("interface");

        sig.append({
            let mut n = Tokens::new();

            n.append(self.name);

            if !self.parameters.is_empty() {
                n.append("<");
                n.append(self.parameters.join(", "));
                n.append(">");
            }

            n
        });

        if !self.extends.is_empty() {
            sig.append(":");
            sig.append(
                self.extends
                    .into_iter()
                    .map(Element::from)
                    .collect::<Tokens<_>>()
                    .join(", "),
            );
        }

        let mut s = Tokens::new();

        if !self.attributes.is_empty() {
            s.push(self.attributes);
        }

        s.push(toks![sig.join_spacing(), " {"]);
        s.nested({
            let mut body = Tokens::new();

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
    use csharp::{local, Interface};
    use tokens::Tokens;
    use Csharp;

    #[test]
    fn test_interface() {
        let mut i = Interface::new("Foo");
        i.parameters.append("T");
        i.extends = vec![local("Super")];

        let t: Tokens<Csharp> = i.into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public interface Foo<T> : Super {\n}"), out);
    }
}
