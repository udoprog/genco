//! Specialization for Python code generation.

use crate::{Cons, Formatter, Lang, Tokens};
use std::collections::BTreeSet;
use std::fmt::{self, Write};

static SEP: &'static str = ".";

/// Python token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Python<'el> {
    /// Module of the imported name.
    module: Option<Cons<'el>>,
    /// Alias of module.
    alias: Option<Cons<'el>>,
    /// Name imported.
    ///
    /// If `None`, last component of module will be used.
    name: Option<Cons<'el>>,
}

impl<'el> fmt::Display for Python<'el> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let has_module = match self.module {
            Some(ref module) => match self.alias {
                Some(ref alias) => {
                    fmt.write_str(alias)?;
                    true
                }
                None => {
                    if let Some(part) = module.split(SEP).last() {
                        fmt.write_str(part)?;
                        true
                    } else {
                        false
                    }
                }
            },
            None => false,
        };

        if let Some(ref name) = self.name {
            if has_module {
                fmt.write_str(SEP)?;
            }

            fmt.write_str(name.as_ref())?;
        }

        Ok(())
    }
}

impl<'el> Python<'el> {
    fn imports(tokens: &Tokens<'el, Self>) -> Option<Tokens<'el, Self>> {
        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            let Python {
                ref module,
                ref alias,
                ..
            } = *custom;

            if let Some(ref module) = *module {
                modules.insert((module.clone(), alias.clone()));
            }
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for (module, alias) in modules {
            let mut s = Tokens::new();

            s.append("import ");
            s.append(module);

            if let Some(alias) = alias {
                s.append(" as ");
                s.append(alias);
            }

            out.push(s);
        }

        Some(out)
    }

    /// Set alias for python element.
    pub fn alias<N: Into<Cons<'el>>>(self, new_alias: N) -> Python<'el> {
        Python {
            alias: Some(new_alias.into()),
            ..self
        }
    }

    /// Set name for python element.
    pub fn name<N: Into<Cons<'el>>>(self, new_name: N) -> Python<'el> {
        Python {
            name: Some(new_name.into()),
            ..self
        }
    }
}

impl<'el> Lang<'el> for Python<'el> {
    type Config = ();

    fn format(&self, out: &mut Formatter, _extra: &mut Self::Config, _level: usize) -> fmt::Result {
        write!(out, "{}", self)
    }

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_char('"')?;

        for c in input.chars() {
            match c {
                '\t' => out.write_str("\\t")?,
                '\u{0007}' => out.write_str("\\b")?,
                '\n' => out.write_str("\\n")?,
                '\r' => out.write_str("\\r")?,
                '\u{0014}' => out.write_str("\\f")?,
                '\'' => out.write_str("\\'")?,
                '"' => out.write_str("\\\"")?,
                '\\' => out.write_str("\\\\")?,
                c => out.write_char(c)?,
            };
        }

        out.write_char('"')?;

        Ok(())
    }

    fn write_file(
        tokens: Tokens<'el, Self>,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks: Tokens<Self> = Tokens::new();

        if let Some(imports) = Self::imports(&tokens) {
            toks.push(imports);
            toks.line_spacing();
        }

        toks.extend(tokens);
        toks.format(out, config, level)
    }
}

/// Setup an imported element.
pub fn imported<'a, M>(module: M) -> Python<'a>
where
    M: Into<Cons<'a>>,
{
    Python {
        module: Some(module.into()),
        alias: None,
        name: None,
    }
}

/// Setup a local element.
pub fn local<'a, N>(name: N) -> Python<'a>
where
    N: Into<Cons<'a>>,
{
    Python {
        module: None,
        alias: None,
        name: Some(name.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{imported, local, Python};
    use crate::quoted::Quoted;
    use crate::tokens::Tokens;

    #[test]
    fn test_string() {
        let mut toks: Tokens<Python> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }

    #[test]
    fn test_imported() {
        let mut toks: Tokens<Python> = Tokens::new();
        toks.push(toks![imported("collections").name("named_tuple")]);
        toks.push(toks![imported("collections")]);
        toks.push(toks![imported("collections")
            .alias("c")
            .name("named_tuple")]);
        toks.push(toks![imported("collections").alias("c")]);

        assert_eq!(
            Ok("import collections\nimport collections as c\n\ncollections.named_tuple\ncollections\nc.named_tuple\nc\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }

    #[test]
    fn test_local() {
        let mut toks: Tokens<Python> = Tokens::new();
        toks.push(toks![local("dict")]);

        assert_eq!(Ok("dict\n"), toks.to_file().as_ref().map(|s| s.as_str()));
    }
}
