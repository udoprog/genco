//! Data structure for constructors

use tokens::Tokens;
use java::Java;
use super::argument::Argument;
use con::Con::Owned;
use cons::Cons;
use super::modifier::Modifier;
use element::Element;

/// Model for Java Constructors.
#[derive(Debug, Clone)]
pub struct Constructor<'el> {
    /// Constructor modifiers.
    pub modifiers: Vec<Modifier>,
    /// Arguments for the constructor.
    pub arguments: Vec<Argument<'el>>,
    /// Body of the constructor.
    pub body: Tokens<'el, Java<'el>>,
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
            body: Tokens::new(),
        }
    }

    /// Push an annotation.
    pub fn annotation<A>(&mut self, annotation: A)
    where
        A: Into<Tokens<'el, Java<'el>>>,
    {
        self.annotations.push(annotation.into());
    }
}

impl<'el> From<(Cons<'el>, Constructor<'el>)> for Tokens<'el, Java<'el>> {
    fn from(value: (Cons<'el>, Constructor<'el>)) -> Tokens<'el, Java<'el>> {
        use self::Element::*;

        let (name, c) = value;

        let args: Vec<Tokens<Java>> = c.arguments.into_iter().map(|a| a.into()).collect();
        let args: Tokens<Java> = args.into();

        let mut sig: Tokens<Java> = Tokens::new();

        if !c.modifiers.is_empty() {
            sig.append(c.modifiers);
            sig.append(" ");
        }

        if !args.is_empty() {
            let sep = toks![",", PushSpacing];
            let args = args.join(sep);

            sig.append(toks![
                name, "(", Nested(Owned(args)), ")",
            ]);
        } else {
            sig.append(toks![name, "()"]);
        }

        let mut s = Tokens::new();

        if !c.annotations.is_empty() {
            s.push(c.annotations);
        }

        s.push(toks![sig, " {"]);
        s.nested(c.body);
        s.push("}");

        s
    }
}

#[cfg(test)]
mod tests {
    use super::Constructor;
    use java::Java;
    use tokens::Tokens;
    use cons::Cons;

    #[test]
    fn test_vec() {
        let c = Constructor::new();
        let t: Tokens<Java> = (Cons::Borrowed("Foo"), c).into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public Foo() {\n}"), out);
    }
}
