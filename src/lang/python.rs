//! Specialization for Python code generation.

use crate::{Cons, Formatter, Lang, LangItem};
use std::collections::BTreeSet;
use std::fmt::{self, Write};

/// Tokens container specialization for Python.
pub type Tokens<'el> = crate::Tokens<'el, Python>;

impl_lang_item!(Type, Python);

static SEP: &'static str = ".";

/// Python token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type {
    /// Module of the imported name.
    module: Option<Cons<'static>>,
    /// Alias of module.
    alias: Option<Cons<'static>>,
    /// Name imported.
    ///
    /// If `None`, last component of module will be used.
    name: Option<Cons<'static>>,
}

impl fmt::Display for Type {
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

impl Type {
    /// Set alias for python element.
    pub fn alias<N: Into<Cons<'static>>>(self, new_alias: N) -> Type {
        Self {
            alias: Some(new_alias.into()),
            ..self
        }
    }

    /// Set name for python element.
    pub fn name<N: Into<Cons<'static>>>(self, new_name: N) -> Type {
        Self {
            name: Some(new_name.into()),
            ..self
        }
    }
}

impl LangItem<Python> for Type {
    fn format(&self, out: &mut Formatter, _extra: &mut (), _level: usize) -> fmt::Result {
        write!(out, "{}", self)
    }

    fn as_import(&self) -> Option<&Self> {
        Some(self)
    }
}

/// Language specialization for Python.
pub struct Python(());

impl Python {
    fn imports<'el>(tokens: &Tokens<'el>) -> Option<Tokens<'el>> {
        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            if let Some(import) = custom.as_import() {
                let Type { module, alias, .. } = import;

                if let Some(ref module) = *module {
                    modules.insert((module.clone(), alias.clone()));
                }
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
}

impl Lang for Python {
    type Config = ();
    type Import = Type;

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
        tokens: Tokens<'_>,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks = Tokens::new();

        if let Some(imports) = Self::imports(&tokens) {
            toks.push(imports);
            toks.line_spacing();
        }

        toks.extend(tokens);
        toks.format(out, config, level)
    }
}

/// Setup an imported element.
pub fn imported<M>(module: M) -> Type
where
    M: Into<Cons<'static>>,
{
    Type {
        module: Some(module.into()),
        alias: None,
        name: None,
    }
}

/// Setup a local element.
pub fn local<N>(name: N) -> Type
where
    N: Into<Cons<'static>>,
{
    Type {
        module: None,
        alias: None,
        name: Some(name.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{imported, local, Tokens};
    use crate as genco;
    use crate::{quote, Ext as _};

    #[test]
    fn test_string() {
        let mut toks = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }

    #[test]
    fn test_imported() {
        let mut toks = Tokens::new();
        toks.push(quote![#(imported("collections").name("named_tuple"))]);
        toks.push(quote![#(imported("collections"))]);
        toks.push(quote![#(imported("collections")
            .alias("c")
            .name("named_tuple"))]);
        toks.push(quote![#(imported("collections").alias("c"))]);

        assert_eq!(
            Ok("import collections\nimport collections as c\n\ncollections.named_tuple\ncollections\nc.named_tuple\nc\n"),
            toks.to_file_string().as_ref().map(|s| s.as_str())
        );
    }

    #[test]
    fn test_local() {
        let mut toks = Tokens::new();
        toks.push(quote![#(local("dict"))]);

        assert_eq!(
            Ok("dict\n"),
            toks.to_file_string().as_ref().map(|s| s.as_str())
        );
    }
}
