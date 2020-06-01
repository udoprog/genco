//! Data structure for enums.

use super::modifier::Modifier;
use crate::cons::Cons;
use crate::csharp::Csharp;
use crate::element::Element;
use crate::into_tokens::IntoTokens;
use crate::tokens::Tokens;

/// Model for Csharp Enums.
#[derive(Debug, Clone)]
pub struct Enum<'el> {
    /// Variants of the enum.
    pub variants: Tokens<'el, Csharp<'el>>,
    /// Enum modifiers.
    pub modifiers: Vec<Modifier>,
    /// What this enum implements.
    pub implements: Vec<Csharp<'el>>,
    /// Attributes for the constructor.
    attributes: Tokens<'el, Csharp<'el>>,
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
            implements: vec![],
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

    /// Name of enum.
    pub fn name(&self) -> Cons<'el> {
        self.name.clone()
    }
}

into_tokens_impl_from!(Enum<'el>, Csharp<'el>);

impl<'el> IntoTokens<'el, Csharp<'el>> for Enum<'el> {
    fn into_tokens(self) -> Tokens<'el, Csharp<'el>> {
        use self::Element::*;

        let mut sig = Tokens::new();

        sig.extend(self.modifiers.into_tokens());
        sig.append("enum");
        sig.append(self.name.clone());

        let mut extends = Tokens::new();

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

            if !self.variants.is_empty() {
                body.append(self.variants.join(toks![",", PushSpacing]));
            }

            body.join_line_spacing()
        });

        s.push("}");

        s
    }
}

#[cfg(test)]
mod tests {
    use super::Enum;
    use crate::csharp::{self, Csharp};
    use crate::tokens::Tokens;

    #[test]
    fn test_vec() {
        let mut c = Enum::new("Foo");

        c.variants.append("FOO(1)");
        c.variants.append("BAR(2)");

        let t: Tokens<Csharp> = c.into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public enum Foo {\n  FOO(1),\n  BAR(2)\n}",), out);
    }

    #[test]
    fn test_implements() {
        let mut c = Enum::new("Foo");

        c.implements = vec![csharp::local("long")];

        c.variants.append("FOO(1)");
        c.variants.append("BAR(2)");

        let t: Tokens<Csharp> = c.into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public enum Foo : long {\n  FOO(1),\n  BAR(2)\n}",), out);
    }
}
