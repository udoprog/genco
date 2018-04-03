//! Data structure for interfaces.

use super::method::Method;
use super::modifier::Modifier;
use cons::Cons;
use into_tokens::IntoTokens;
use java::Java;
use tokens::Tokens;

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
    pub extends: Tokens<'el, Java<'el>>,
    /// Generic parameters.
    pub parameters: Tokens<'el, Java<'el>>,
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
            extends: Tokens::new(),
            parameters: Tokens::new(),
            annotations: Tokens::new(),
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

    /// Name of interface.
    pub fn name(&self) -> Cons<'el> {
        self.name.clone()
    }
}

into_tokens_impl_from!(Interface<'el>, Java<'el>);

impl<'el> IntoTokens<'el, Java<'el>> for Interface<'el> {
    fn into_tokens(self) -> Tokens<'el, Java<'el>> {
        let mut sig = Tokens::new();

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
            sig.append("extends");
            sig.append(self.extends.join(", "));
        }

        let mut s = Tokens::new();

        if !self.annotations.is_empty() {
            s.push(self.annotations);
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
    use super::Interface;
    use java::Java;
    use java::local;
    use tokens::Tokens;

    #[test]
    fn test_vec() {
        let mut i = Interface::new("Foo");
        i.parameters.append("T");
        i.extends = local("Super").into();

        let t: Tokens<Java> = i.into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public interface Foo<T> extends Super {\n}"), out);
    }
}
