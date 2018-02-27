//! Data structure for methods.

use {Cons, IntoTokens, Tokens};
use java::{Argument, BlockComment, Java, Modifier, VOID};

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
    /// Generic parameters.
    pub parameters: Tokens<'el, Java<'el>>,
    /// Comments associated with this method.
    pub comments: Vec<Cons<'el>>,
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
            parameters: Tokens::new(),
            comments: Vec::new(),
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

        sig.push_unless_empty(BlockComment(self.comments));

        if !self.modifiers.is_empty() {
            sig.append(self.modifiers);
            sig.append(" ");
        }

        sig.append(toks![self.returns, " ", self.name]);

        if !self.parameters.is_empty() {
            sig.append("<");
            sig.append(self.parameters.join(", "));
            sig.append(">");
        }

        sig.append(toks!["(", args.join(", "), ")"]);

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

    fn build_method() -> Method<'static> {
        let mut c = Method::new("foo");
        c.parameters.append("T");
        c
    }

    #[test]
    fn test_with_comments() {
        let mut c = build_method();
        c.comments.push("Hello World".into());
        let t = Tokens::from(c);
        assert_eq!(
            Ok(String::from(
                "/**\n * Hello World\n */\npublic void foo<T>();",
            )),
            t.to_string()
        );
    }

    #[test]
    fn test_no_comments() {
        let t = Tokens::from(build_method());
        assert_eq!(Ok(String::from("public void foo<T>();")), t.to_string());
    }
}
