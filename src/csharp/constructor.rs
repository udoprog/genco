//! Data structure for constructors

use super::argument::Argument;
use super::modifier::Modifier;
use con_::Con::Owned;
use cons::Cons;
use element::Element;
use into_tokens::IntoTokens;
use csharp::Csharp;
use tokens::Tokens;

/// Model for Csharp Constructors.
#[derive(Debug, Clone)]
pub struct Constructor<'el> {
    /// Constructor modifiers.
    pub modifiers: Vec<Modifier>,
    /// Arguments for the constructor.
    pub arguments: Vec<Argument<'el>>,
    /// Body of the constructor.
    pub body: Tokens<'el, Csharp<'el>>,
    /// Base call
    pub base: Option<Tokens<'el, Csharp<'el>>>,
    /// attributes for the constructor.
    attributes: Tokens<'el, Csharp<'el>>,
}

impl<'el> Constructor<'el> {
    /// Build a new empty constructor.
    pub fn new() -> Constructor<'el> {
        Constructor {
            modifiers: vec![Modifier::Public],
            arguments: Vec::new(),
            body: Tokens::new(),
            base: None,
            attributes: Tokens::new(),
        }
    }

    /// Push a attribute.
    pub fn attribute<T>(&mut self, attribute: T)
    where
        T: IntoTokens<'el, Csharp<'el>>,
    {
        self.attributes.push(attribute.into_tokens());
    }
}

into_tokens_impl_from!((Cons<'el>, Constructor<'el>), Csharp<'el>);

impl<'el> IntoTokens<'el, Csharp<'el>> for (Cons<'el>, Constructor<'el>) {
    fn into_tokens(self) -> Tokens<'el, Csharp<'el>> {
        use self::Element::*;

        let (name, mut c) = self;

        let args: Vec<Tokens<Csharp>> = c.arguments.into_iter().map(|a| a.into_tokens()).collect();
        let args: Tokens<Csharp> = args.into_tokens();

        let mut sig: Tokens<Csharp> = Tokens::new();

        c.modifiers.sort();
        sig.extend(c.modifiers.into_iter().map(Into::into));

        if !args.is_empty() {
            let sep = toks![",", PushSpacing];
            let args = args.join(sep);

            sig.append(toks![name, "(", Nested(Owned(args)), ")",]);
        } else {
            sig.append(toks![name, "()"]);
        }

        if let Some(base) = c.base {
            sig.append(":");
            sig.append(base);
        }

        let mut s = Tokens::new();

        if !c.attributes.is_empty() {
            s.push(c.attributes);
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
    use cons::Cons;
    use csharp::Csharp;
    use tokens::Tokens;

    #[test]
    fn test_vec() {
        let c = Constructor::new();
        let t: Tokens<Csharp> = (Cons::Borrowed("Foo"), c).into();

        let s = t.to_string();
        let out = s.as_ref().map(|s| s.as_str());
        assert_eq!(Ok("public Foo() {\n}"), out);
    }
}
