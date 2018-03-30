//! Specialization for Rust code generation.

use std::collections::BTreeSet;
use std::fmt::{self, Write};
use {Cons, Custom, Formatter, IntoTokens, Tokens};

static SEP: &'static str = "::";

/// A name.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Name<'el> {
    /// Name  of class.
    name: Cons<'el>,
    /// Arguments of the class.
    arguments: Vec<Rust<'el>>,
}

impl<'el> Name<'el> {
    /// Format the name.
    fn format(&self, out: &mut Formatter, extra: &mut (), level: usize) -> fmt::Result {
        out.write_str(self.name.as_ref())?;

        if !self.arguments.is_empty() {
            let mut it = self.arguments.iter().peekable();

            out.write_str("<")?;

            while let Some(n) = it.next() {
                n.format(out, extra, level + 1)?;

                if it.peek().is_some() {
                    out.write_str(", ")?;
                }
            }

            out.write_str(">")?;
        }

        Ok(())
    }

    /// Add generic arguments to the given type.
    pub fn with_arguments(&self, arguments: Vec<Rust<'el>>) -> Name<'el> {
        Name {
            name: self.name.clone(),
            arguments: arguments,
        }
    }
}

impl<'el> From<Cons<'el>> for Name<'el> {
    fn from(value: Cons<'el>) -> Self {
        Name {
            name: value,
            arguments: vec![],
        }
    }
}

/// Rust token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Rust<'el> {
    /// Module of the imported name.
    module: Option<Cons<'el>>,
    /// Alias of module.
    alias: Option<Cons<'el>>,
    /// Name imported.
    name: Name<'el>,
}

into_tokens_impl_from!(Rust<'el>, Rust<'el>);
into_tokens_impl_from!(&'el Rust<'el>, Rust<'el>);

impl<'el> Rust<'el> {
    fn imports<'a>(tokens: &'a Tokens<'a, Self>) -> Option<Tokens<'a, Self>> {
        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            if let Some(module) = custom.module.as_ref() {
                modules.insert((module.as_ref(), custom.alias.as_ref()));
            }
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for (module, alias) in modules {
            let mut s = Tokens::new();

            s.append("use ");
            s.append(module);

            if let Some(alias) = alias {
                s.append(" as ");
                s.append(alias.as_ref());
            }

            s.append(";");

            out.push(s);
        }

        Some(out)
    }

    /// Alias the given type.
    pub fn alias<A: Into<Cons<'el>>>(self, alias: A) -> Rust<'el> {
        Rust {
            alias: Some(alias.into()),
            ..self
        }
    }

    /// Add generic arguments to the given type.
    pub fn with_arguments(&self, arguments: Vec<Rust<'el>>) -> Rust<'el> {
        Rust {
            module: self.module.clone(),
            name: self.name.with_arguments(arguments),
            alias: self.alias.clone(),
        }
    }
}

impl<'el> Custom for Rust<'el> {
    type Extra = ();

    fn format(&self, out: &mut Formatter, extra: &mut Self::Extra, level: usize) -> fmt::Result {
        if let Some(alias) = self.alias.as_ref() {
            out.write_str(alias)?;
            out.write_str(SEP)?;
        } else if let Some(part) = self.module.as_ref().and_then(|m| m.split(SEP).last()) {
            out.write_str(part)?;
            out.write_str(SEP)?;
        }

        self.name.format(out, extra, level)
    }

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_char('"')?;

        for c in input.chars() {
            match c {
                '\t' => out.write_str("\\t")?,
                '\n' => out.write_str("\\n")?,
                '\r' => out.write_str("\\r")?,
                '\'' => out.write_str("\\'")?,
                '"' => out.write_str("\\\"")?,
                '\\' => out.write_str("\\\\")?,
                c => out.write_char(c)?,
            };
        }

        out.write_char('"')?;
        Ok(())
    }

    fn write_file<'a>(
        tokens: Tokens<'a, Self>,
        out: &mut Formatter,
        extra: &mut Self::Extra,
        level: usize,
    ) -> fmt::Result {
        let mut toks: Tokens<Self> = Tokens::new();

        if let Some(imports) = Self::imports(&tokens) {
            toks.push(imports);
        }

        toks.push_ref(&tokens);
        toks.join_line_spacing().format(out, extra, level)
    }
}

/// Setup an imported element.
pub fn imported<'a, M, N>(module: M, name: N) -> Rust<'a>
where
    M: Into<Cons<'a>>,
    N: Into<Cons<'a>>,
{
    Rust {
        module: Some(module.into()),
        alias: None,
        name: Name::from(name.into()),
    }
}

/// Setup a local element.
pub fn local<'a, N>(name: N) -> Rust<'a>
where
    N: Into<Cons<'a>>,
{
    Rust {
        module: None,
        alias: None,
        name: Name::from(name.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{imported, local};
    use quoted::Quoted;
    use rust::Rust;
    use tokens::Tokens;

    #[test]
    fn test_string() {
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.append("hello \n world".quoted());
        let res = toks.to_string();

        assert_eq!(Ok("\"hello \\n world\""), res.as_ref().map(|s| s.as_str()));
    }

    #[test]
    fn test_imported() {
        let dbg = imported("std::fmt", "Debug");
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("use std::fmt;\n\nfmt::Debug\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }

    #[test]
    fn test_imported_alias() {
        let dbg = imported("std::fmt", "Debug").alias("dbg");
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("use std::fmt as dbg;\n\ndbg::Debug\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }

    #[test]
    fn test_imported_with_arguments() {
        let dbg = imported("std::fmt", "Debug")
            .alias("dbg")
            .with_arguments(vec![local("T"), local("U")]);
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("use std::fmt as dbg;\n\ndbg::Debug<T, U>\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
