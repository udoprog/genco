//! Specialization for Rust code generation.

use crate::{Cons, Custom, Formatter, IntoTokens, Tokens};
use std::collections::BTreeSet;
use std::fmt::{self, Write};
use std::rc::Rc;

static SEP: &'static str = "::";

/// The inferred reference.
#[derive(Debug, Clone, Copy)]
pub struct Ref;

/// The static reference.
#[derive(Debug, Clone, Copy)]
pub struct StaticRef;

/// Reference information about a name.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Reference<'el> {
    /// An anonymous reference.
    Ref,
    /// A static reference.
    StaticRef,
    /// A named reference.
    Named(Cons<'el>),
}

impl From<Ref> for Reference<'static> {
    fn from(_: Ref) -> Self {
        Reference::Ref
    }
}

impl From<StaticRef> for Reference<'static> {
    fn from(_: StaticRef) -> Self {
        Reference::StaticRef
    }
}

impl From<Rc<String>> for Reference<'static> {
    fn from(value: Rc<String>) -> Self {
        Reference::Named(Cons::from(value))
    }
}

impl<'el> From<&'el str> for Reference<'el> {
    fn from(value: &'el str) -> Self {
        Reference::Named(Cons::from(value))
    }
}

/// A name.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Name<'el> {
    reference: Option<Reference<'el>>,
    /// Name  of class.
    name: Cons<'el>,
    /// Arguments of the class.
    arguments: Vec<Rust<'el>>,
}

impl<'el> Name<'el> {
    /// Format the name.
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        if let Some(reference) = self.reference.as_ref() {
            match *reference {
                Reference::StaticRef => {
                    out.write_str("&'static ")?;
                }
                Reference::Named(ref name) => {
                    out.write_str("&'")?;
                    out.write_str(name.as_ref())?;
                    out.write_str(" ")?;
                }
                Reference::Ref => {
                    out.write_str("&")?;
                }
            }
        }

        out.write_str(self.name.as_ref())?;

        if !self.arguments.is_empty() {
            let mut it = self.arguments.iter().peekable();

            out.write_str("<")?;

            while let Some(n) = it.next() {
                n.format(out, config, level + 1)?;

                if it.peek().is_some() {
                    out.write_str(", ")?;
                }
            }

            out.write_str(">")?;
        }

        Ok(())
    }

    /// Add generic arguments to the given type.
    pub fn with_arguments(self, arguments: Vec<Rust<'el>>) -> Name<'el> {
        Name {
            arguments: arguments,
            ..self
        }
    }

    /// Create a name with the given reference.
    pub fn reference<R: Into<Reference<'el>>>(self, reference: R) -> Name<'el> {
        Name {
            reference: Some(reference.into()),
            ..self
        }
    }
}

impl<'el> From<Cons<'el>> for Name<'el> {
    fn from(value: Cons<'el>) -> Self {
        Name {
            reference: None,
            name: value,
            arguments: vec![],
        }
    }
}

/// Language configuration for Rust.
#[derive(Debug)]
pub struct Config {
    indentation: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { indentation: 4 }
    }
}

impl Config {
    /// Configure the indentation for Rust.
    pub fn with_indentation(self, indentation: usize) -> Self {
        Self { indentation }
    }
}

impl crate::Config for Config {
    fn indentation(&mut self) -> usize {
        self.indentation
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
    /// Qualified import.
    qualified: bool,
}

into_tokens_impl_from!(Rust<'el>, Rust<'el>);
into_tokens_impl_from!(&'el Rust<'el>, Rust<'el>);

impl<'el> Rust<'el> {
    fn walk_custom<'a, 'b: 'a>(
        custom: &'a Rust<'b>,
        modules: &mut BTreeSet<(Cons<'a>, Option<&'a Cons<'b>>)>,
    ) {
        if let Some(module) = custom.module.as_ref() {
            if custom.qualified || custom.alias.is_some() {
                let module = Cons::from(format!("{}::{}", module, custom.name.name.as_ref()));
                modules.insert((module, custom.alias.as_ref()));
            } else {
                modules.insert((Cons::from(module.as_ref()), custom.alias.as_ref()));
            }
        }

        for arg in &custom.name.arguments {
            Self::walk_custom(arg, modules);
        }
    }

    fn imports<'a>(tokens: &'a Tokens<'a, Self>) -> Option<Tokens<'a, Self>> {
        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            Self::walk_custom(&custom, &mut modules);
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for (module, alias) in modules {
            if module.split("::").count() == 1 {
                continue;
            }

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
    pub fn with_arguments(self, arguments: Vec<Rust<'el>>) -> Rust<'el> {
        Rust {
            name: self.name.with_arguments(arguments),
            ..self
        }
    }

    /// Change to be a qualified import.
    pub fn qualified(self) -> Rust<'el> {
        Rust {
            qualified: true,
            ..self
        }
    }

    /// Make the type a reference.
    pub fn reference<R: Into<Reference<'el>>>(self, reference: R) -> Rust<'el> {
        Rust {
            module: self.module,
            name: self.name.reference(reference),
            alias: self.alias,
            qualified: self.qualified,
        }
    }
}

impl<'el> Custom for Rust<'el> {
    type Config = Config;

    fn format(&self, out: &mut Formatter, config: &mut Self::Config, level: usize) -> fmt::Result {
        if let Some(alias) = self.alias.as_ref() {
            out.write_str(alias)?;
            out.write_str(SEP)?;
        } else if !self.qualified {
            if let Some(part) = self.module.as_ref().and_then(|m| m.split(SEP).last()) {
                out.write_str(part)?;
                out.write_str(SEP)?;
            }
        }

        self.name.format(out, config, level)
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
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks: Tokens<Self> = Tokens::new();

        if let Some(imports) = Self::imports(&tokens) {
            toks.push(imports);
        }

        toks.push_ref(&tokens);
        toks.join_line_spacing().format(out, config, level)
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
        qualified: false,
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
        qualified: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{imported, local};
    use crate::quoted::Quoted;
    use crate::rust::Rust;
    use crate::tokens::Tokens;

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
        let dbg = imported("std::fmt", "Debug");
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("use std::fmt;\n\nfmt::Debug\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }

    #[test]
    fn test_imported_with_arguments() {
        let dbg = imported("std::fmt", "Debug").with_arguments(vec![local("T"), local("U")]);
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("use std::fmt;\n\nfmt::Debug<T, U>\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
