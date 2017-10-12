//! Data structure for classes.

use tokens::Tokens;
use java::Java;
use cons::Cons;
use super::modifier::Modifier;
use super::method::Method;
use super::constructor::Constructor;
use super::field::Field;
use element::Element;

/// Model for Java Classs.
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
    pub body: Tokens<'el, Java<'el>>,
    /// What this class extends.
    pub extends: Option<Java<'el>>,
    /// What this class implements.
    pub implements: Vec<Java<'el>>,
    /// Annotations for the constructor.
    annotations: Tokens<'el, Java<'el>>,
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

    /// Name of class.
    pub fn name(&self) -> Cons<'el> {
        self.name.clone()
    }
}

impl<'el> From<Class<'el>> for Tokens<'el, Java<'el>> {
    fn from(c: Class<'el>) -> Tokens<'el, Java<'el>> {
        let mut sig = Tokens::new();

        if !c.modifiers.is_empty() {
            sig.append(c.modifiers);
            sig.append(" ");
        }

        sig.append("class ");
        sig.append(c.name.clone());

        if let Some(extends) = c.extends {
            sig.append("extends ");
            sig.append(extends);
        }

        if !c.implements.is_empty() {
            let implements: Tokens<_> = c.implements
                .into_iter()
                .map::<Element<_>, _>(Into::into)
                .collect();

            sig.append("implements ");
            sig.append(implements.join(", "));
        }

        let mut s = Tokens::new();

        if !c.annotations.is_empty() {
            s.push(c.annotations);
        }

        s.push(toks![sig, " {"]);

        s.nested({
            let mut body = Tokens::new();

            if !c.fields.is_empty() {
                let mut fields = Tokens::new();

                for field in c.fields {
                    fields.push(toks![field, ";"]);
                }

                body.push(fields);
            }

            if !c.constructors.is_empty() {
                for constructor in c.constructors {
                    body.push((c.name.clone(), constructor));
                }
            }

            if !c.methods.is_empty() {
                for method in c.methods {
                    body.push(method);
                }
            }

            body.extend(c.body);
            body.join_line_spacing()
        });

        s.push("}");

        s
    }
}

#[cfg(test)]
mod tests {
    use super::Class;
    use java::Java;
    use tokens::Tokens;

    #[test]
    fn test_vec() {
        let c = Class::new("Foo");
        let t: Tokens<Java> = c.into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public class Foo {\n}"), out);
    }
}
