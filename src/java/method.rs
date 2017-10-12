//! Data structure for methods.

use tokens::Tokens;
use java::Java;
use super::argument::Argument;
use super::modifier::Modifier;
use super::VOID;
use cons::Cons;

/// Model for Java Methods.
#[derive(Debug, Clone)]
pub struct Method<'el> {
    /// Method modifiers.
    pub modifiers: Vec<Modifier>,
    /// Arguments for the constructor.
    pub arguments: Vec<Argument<'el>>,
    /// Body of the constructor.
    pub body: Tokens<'el, Java<'el>>,
    /// Return type.
    pub returns: Java<'el>,
    /// Annotations for the constructor.
    annotations: Tokens<'el, Java<'el>>,
    /// Name of the method.
    name: Cons<'el>,
}

impl<'el> Method<'el> {
    /// Build a new empty constructor.
    pub fn new<N>(name: N) -> Method<'el>
    where
        N: Into<Cons<'el>>,
    {
        Method {
            modifiers: vec![Modifier::Public],
            arguments: Vec::new(),
            body: Tokens::new(),
            returns: VOID,
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
}

impl<'el> From<Method<'el>> for Tokens<'el, Java<'el>> {
    fn from(m: Method<'el>) -> Tokens<'el, Java<'el>> {
        let args: Vec<Tokens<Java>> = m.arguments.into_iter().map(|a| a.into()).collect();
        let args: Tokens<Java> = args.into();

        let mut sig = Tokens::new();

        if !m.modifiers.is_empty() {
            sig.append(m.modifiers);
            sig.append(" ");
        }

        sig.append(toks![m.returns, " ", m.name, "(", args.join_spacing(), ")"]);

        let mut s = Tokens::new();

        if !m.annotations.is_empty() {
            s.push(m.annotations);
        }

        if m.body.is_empty() {
            s.push(toks![sig, ";"]);
        } else {
            s.push(toks![sig, " {"]);
            s.nested(m.body.join_line_spacing());
            s.push("}");
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::Method;
    use tokens::Tokens;

    #[test]
    fn test_empty_body() {
        let c = Method::new("foo");
        let t: Tokens<_> = c.into();
        assert_eq!(Ok(String::from("public void foo();")), t.to_string());
    }
}
