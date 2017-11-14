//! Data structure for methods.

use tokens::Tokens;
use java::Java;
use super::argument::Argument;
use super::modifier::Modifier;
use super::VOID;
use cons::Cons;
use into_tokens::IntoTokens;

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
        use self::Modifier::*;

        Method {
            modifiers: vec![Public],
            arguments: vec![],
            body: Tokens::new(),
            returns: VOID,
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

    /// Name of method.
    pub fn name(&self) -> Cons<'el> {
        self.name.clone()
    }
}

into_tokens_impl_from!(Method<'el>, Java<'el>);

impl<'el> IntoTokens<'el, Java<'el>> for Method<'el> {
    fn into_tokens(self) -> Tokens<'el, Java<'el>> {
        let args: Vec<Tokens<Java>> = self.arguments
            .into_iter()
            .map(IntoTokens::into_tokens)
            .collect();

        let args: Tokens<Java> = args.into_tokens();

        let mut sig = Tokens::new();

        if !self.modifiers.is_empty() {
            sig.append(self.modifiers);
            sig.append(" ");
        }

        sig.append(toks![
            self.returns,
            " ",
            self.name,
            "(",
            args.join(", "),
            ")",
        ]);

        let mut s = Tokens::new();

        if !self.annotations.is_empty() {
            s.push(self.annotations);
        }

        if self.body.is_empty() {
            s.push(toks![sig, ";"]);
        } else {
            s.push(toks![sig, " {"]);
            s.nested(self.body);
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
