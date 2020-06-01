//! Data structure for constructors

use super::argument::Argument;
use super::modifier::Modifier;
use crate::con_::Con::Owned;
use crate::cons::Cons;
use crate::element::Element;
use crate::into_tokens::IntoTokens;
use crate::java::Java;
use crate::tokens::Tokens;

/// Model for Java Constructors.
#[derive(Debug, Clone)]
pub struct Constructor<'el> {
    /// Constructor modifiers.
    pub modifiers: Vec<Modifier>,
    /// Arguments for the constructor.
    pub arguments: Vec<Argument<'el>>,
    /// Body of the constructor.
    pub body: Tokens<'el, Java<'el>>,
    /// Exception thrown by the constructor.
    pub throws: Option<Tokens<'el, Java<'el>>>,
    /// Annotations for the constructor.
    annotations: Tokens<'el, Java<'el>>,
}

impl<'el> Constructor<'el> {
    /// Build a new empty constructor.
    pub fn new() -> Constructor<'el> {
        Constructor {
            modifiers: vec![Modifier::Public],
            annotations: Tokens::new(),
            arguments: Vec::new(),
            throws: None,
            body: Tokens::new(),
        }
    }

    /// Push an annotation.
    pub fn annotation<A>(&mut self, annotation: A)
    where
        A: IntoTokens<'el, Java<'el>>,
    {
        self.annotations.push(annotation.into_tokens());
    }
}

into_tokens_impl_from!((Cons<'el>, Constructor<'el>), Java<'el>);

impl<'el> IntoTokens<'el, Java<'el>> for (Cons<'el>, Constructor<'el>) {
    fn into_tokens(self) -> Tokens<'el, Java<'el>> {
        use self::Element::*;

        let (name, mut c) = self;

        let args: Vec<Tokens<Java>> = c.arguments.into_iter().map(|a| a.into_tokens()).collect();
        let args: Tokens<Java> = args.into_tokens();

        let mut sig: Tokens<Java> = Tokens::new();

        c.modifiers.sort();
        sig.extend(c.modifiers.into_iter().map(Into::into));

        if !args.is_empty() {
            let sep = toks![",", PushSpacing];
            let args = args.join(sep);

            sig.append(toks![name, "(", Nested(Owned(args)), ")",]);
        } else {
            sig.append(toks![name, "()"]);
        }

        if let Some(throws) = c.throws {
            sig.append("throws");
            sig.append(throws);
        }

        let mut s = Tokens::new();

        if !c.annotations.is_empty() {
            s.push(c.annotations);
        }

        s.push(toks![sig.join_spacing(), " {"]);
        s.nested(c.body);
        s.push("}");

        s
    }
}

#[cfg(test)]
mod tests {
    use super::Constructor;
    use crate::cons::Cons;
    use crate::java::Java;
    use crate::tokens::Tokens;

    #[test]
    fn test_vec() {
        let c = Constructor::new();
        let t: Tokens<Java> = (Cons::Borrowed("Foo"), c).into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public Foo() {\n}"), out);
    }

    #[test]
    fn test_throws() {
        let mut c = Constructor::new();
        c.throws = Some("Exception".into());
        let t: Tokens<Java> = (Cons::Borrowed("Foo"), c).into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public Foo() throws Exception {\n}"), out);
    }
}
