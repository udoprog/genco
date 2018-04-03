//! Data structure for methods.

use csharp::{Argument, BlockComment, Csharp, Modifier};
use {Cons, IntoTokens, Tokens};

/// Model for Csharp Methods.
#[derive(Debug, Clone)]
pub struct Method<'el> {
    /// Method modifiers.
    pub modifiers: Vec<Modifier>,
    /// Arguments for the constructor.
    pub arguments: Vec<Argument<'el>>,
    /// Body of the constructor.
    pub body: Tokens<'el, Csharp<'el>>,
    /// Return type.
    pub returns: Csharp<'el>,
    /// Generic parameters.
    pub parameters: Tokens<'el, Csharp<'el>>,
    /// Comments associated with this method.
    pub comments: Vec<Cons<'el>>,
    /// attributes for the constructor.
    attributes: Tokens<'el, Csharp<'el>>,
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
            returns: Csharp::Void,
            parameters: Tokens::new(),
            comments: Vec::new(),
            attributes: Tokens::new(),
            name: name.into(),
        }
    }

    /// Push a attribute.
    pub fn attribute<T>(&mut self, attribute: T)
    where
        T: IntoTokens<'el, Csharp<'el>>,
    {
        self.attributes.push(attribute.into_tokens());
    }

    /// Name of method.
    pub fn name(&self) -> Cons<'el> {
        self.name.clone()
    }
}

into_tokens_impl_from!(Method<'el>, Csharp<'el>);

impl<'el> IntoTokens<'el, Csharp<'el>> for Method<'el> {
    fn into_tokens(self) -> Tokens<'el, Csharp<'el>> {
        let mut sig = Tokens::new();

        sig.extend(self.modifiers.into_tokens());
        sig.append(self.returns);
        sig.append({
            let mut n = Tokens::new();
            n.append(self.name);

            if !self.parameters.is_empty() {
                n.append("<");
                n.append(self.parameters.join(", "));
                n.append(">");
            }

            let args: Vec<Tokens<Csharp>> = self.arguments
                .into_iter()
                .map(IntoTokens::into_tokens)
                .collect();

            let args: Tokens<Csharp> = args.into_tokens();

            n.append(toks!["(", args.join(", "), ")"]);

            n
        });

        let mut s = Tokens::new();

        s.push_unless_empty(BlockComment(self.comments));
        s.push_unless_empty(self.attributes);

        let sig = sig.join_spacing();

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
    use csharp::Method;
    use tokens::Tokens;

    fn build_method() -> Method<'static> {
        let mut c = Method::new("Foo");
        c.parameters.append("T");
        c
    }

    #[test]
    fn test_with_comments() {
        let mut c = build_method();
        c.comments.push("Hello World".into());
        let t = Tokens::from(c);
        assert_eq!(
            Ok(String::from("/// Hello World\npublic void Foo<T>();",)),
            t.to_string()
        );
    }

    #[test]
    fn test_no_comments() {
        let t = Tokens::from(build_method());
        assert_eq!(Ok(String::from("public void Foo<T>();")), t.to_string());
    }
}
