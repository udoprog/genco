//! Data structure for enums.

use tokens::Tokens;
use java::Java;
use cons::Cons;
use super::modifier::Modifier;
use super::method::Method;
use super::constructor::Constructor;
use super::field::Field;
use element::Element;

/// Model for Java Enums.
#[derive(Debug, Clone)]
pub struct Enum<'el> {
    /// Variants of the enum.
    pub variants: Tokens<'el, Java<'el>>,
    /// Enum modifiers.
    pub modifiers: Vec<Modifier>,
    /// Declared methods.
    pub fields: Vec<Field<'el>>,
    /// Declared methods.
    pub constructors: Vec<Constructor<'el>>,
    /// Declared methods.
    pub methods: Vec<Method<'el>>,
    /// Extra body (at end of enum).
    pub body: Tokens<'el, Java<'el>>,
    /// What this enum extends.
    pub extends: Option<Tokens<'el, Java<'el>>>,
    /// What this enum implements.
    pub implements: Vec<Tokens<'el, Java<'el>>>,
    /// Annotations for the constructor.
    annotations: Tokens<'el, Java<'el>>,
    /// Name of enum.
    name: Cons<'el>,
}

impl<'el> Enum<'el> {
    /// Build a new empty interface.
    pub fn new<N>(name: N) -> Enum<'el>
    where
        N: Into<Cons<'el>>,
    {
        Enum {
            variants: Tokens::new(),
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

    /// Name of enum.
    pub fn name(&self) -> Cons<'el> {
        self.name.clone()
    }
}

impl<'el> From<Enum<'el>> for Tokens<'el, Java<'el>> {
    fn from(e: Enum<'el>) -> Tokens<'el, Java<'el>> {
        use self::Element::*;

        let mut sig = Tokens::new();

        if !e.modifiers.is_empty() {
            sig.append(e.modifiers);
            sig.append(" ");
        }

        sig.append("enum ");
        sig.append(e.name.clone());

        if let Some(extends) = e.extends {
            sig.append("extends ");
            sig.append(extends);
        }

        if !e.implements.is_empty() {
            let implements: Tokens<_> = e.implements
                .into_iter()
                .map::<Element<_>, _>(Into::into)
                .collect();

            sig.append("implements ");
            sig.append(implements.join(", "));
        }

        let mut s = Tokens::new();

        if !e.annotations.is_empty() {
            s.push(e.annotations);
        }

        s.push(toks![sig, " {"]);

        s.nested({
            let mut body = Tokens::new();

            if !e.variants.is_empty() {
                let sep = toks![",", PushSpacing];
                let mut variants = e.variants.join(sep);
                variants.append(";");
                body.append(variants);
            }

            if !e.fields.is_empty() {
                let mut fields = Tokens::new();

                for field in e.fields {
                    fields.push(toks![field, ";"]);
                }

                body.push(fields);
            }

            if !e.constructors.is_empty() {
                for constructor in e.constructors {
                    body.push((e.name.clone(), constructor));
                }
            }

            if !e.methods.is_empty() {
                for method in e.methods {
                    body.push(method);
                }
            }

            body.extend(e.body);
            body.join_line_spacing()
        });

        s.push("}");

        s
    }
}

#[cfg(test)]
mod tests {
    use super::Enum;
    use java::Java;
    use tokens::Tokens;

    #[test]
    fn test_vec() {
        let mut c = Enum::new("Foo");
        c.body.push("hello");
        c.body.push("world");

        c.variants.append("FOO(1)");
        c.variants.append("BAR(2)");

        let t: Tokens<Java> = c.into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(
            Ok(
                "public enum Foo {\n  FOO(1),\n  BAR(2);\n\n  hello\n\n  world\n}",
            ),
            out
        );
    }
}
