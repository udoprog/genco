//! Data structure for enums.

use super::constructor::Constructor;
use super::field::Field;
use super::method::Method;
use super::modifier::Modifier;
use crate::cons::Cons;
use crate::element::Element;
use crate::into_tokens::IntoTokens;
use crate::java::Java;
use crate::tokens::Tokens;

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
        A: IntoTokens<'el, Java<'el>>,
    {
        self.annotations.push(annotation.into_tokens());
    }

    /// Name of enum.
    pub fn name(&self) -> Cons<'el> {
        self.name.clone()
    }
}

into_tokens_impl_from!(Enum<'el>, Java<'el>);

impl<'el> IntoTokens<'el, Java<'el>> for Enum<'el> {
    fn into_tokens(self) -> Tokens<'el, Java<'el>> {
        use self::Element::*;

        let mut sig = Tokens::new();

        sig.extend(self.modifiers.into_tokens());

        sig.append("enum");
        sig.append(self.name.clone());

        if let Some(extends) = self.extends {
            sig.append("extends");
            sig.append(extends);
        }

        if !self.implements.is_empty() {
            let implements: Tokens<_> = self
                .implements
                .into_iter()
                .map::<Element<_>, _>(Into::into)
                .collect();

            sig.append("implements");
            sig.append(implements.join(", "));
        }

        let mut s = Tokens::new();

        if !self.annotations.is_empty() {
            s.push(self.annotations);
        }

        s.push(toks![sig.join_spacing(), " {"]);

        s.nested({
            let mut body = Tokens::new();

            if !self.variants.is_empty() {
                let sep = toks![",", PushSpacing];
                let mut variants = self.variants.join(sep);
                variants.append(";");
                body.append(variants);
            } else {
                // Required for _all_ enums.
                body.append(";");
            }

            if !self.fields.is_empty() {
                let mut fields = Tokens::new();

                for field in self.fields {
                    fields.push(toks![field, ";"]);
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
    use super::Enum;
    use crate::java::Java;
    use crate::tokens::Tokens;

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
            Ok("public enum Foo {\n  FOO(1),\n  BAR(2);\n\n  hello\n\n  world\n}",),
            out
        );
    }
}
