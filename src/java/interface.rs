//! Data structure for interfaces.

use tokens::Tokens;
use java::Java;
use cons::Cons;
use super::modifier::Modifier;
use super::method::Method;

/// Model for Java Interfaces.
#[derive(Debug, Clone)]
pub struct Interface<'el> {
    /// Interface modifiers.
    pub modifiers: Vec<Modifier>,
    /// Declared methods.
    pub methods: Vec<Method<'el>>,
    /// Extra body (added to end of interface).
    pub body: Tokens<'el, Java<'el>>,
    /// What this interface extends.
    pub extends: Option<Tokens<'el, Java<'el>>>,
    /// Annotations for the constructor.
    annotations: Tokens<'el, Java<'el>>,
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
            extends: None,
            annotations: Tokens::new(),
            name: name.into(),
        }
    }

    /// Push an annotation.
    pub fn annotation<A>(&mut self, annotation: A)
    where
        A: Into<Tokens<'el, Java<'el>>>,
    {
        self.annotations.push(annotation.into());
    }

    /// Name of interface.
    pub fn name(&self) -> Cons<'el> {
        self.name.clone()
    }
}

impl<'el> From<Interface<'el>> for Tokens<'el, Java<'el>> {
    fn from(i: Interface<'el>) -> Tokens<'el, Java<'el>> {
        let mut sig = Tokens::new();

        if !i.modifiers.is_empty() {
            sig.append(i.modifiers);
            sig.append(" ");
        }

        sig.append("interface ");
        sig.append(i.name);

        if let Some(extends) = i.extends {
            sig.append("extends ");
            sig.append(extends);
        }

        let mut s = Tokens::new();

        if !i.annotations.is_empty() {
            s.push(i.annotations);
        }

        s.push(toks![sig, " {"]);
        s.nested({
            let mut body = Tokens::new();

            if !i.methods.is_empty() {
                for method in i.methods {
                    body.push(method);
                }
            }

            body.extend(i.body);
            body.join_line_spacing()
        });
        s.push("}");

        s
    }
}

#[cfg(test)]
mod tests {
    use super::Interface;
    use java::Java;
    use tokens::Tokens;

    #[test]
    fn test_vec() {
        let i = Interface::new("Foo");
        let t: Tokens<Java> = i.into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public interface Foo {\n}"), out);
    }
}
