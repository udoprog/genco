//! Specialization for Python code generation.

use super::custom::Custom;
use super::formatter::Formatter;
use super::into_tokens::IntoTokens;
use super::tokens::Tokens;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt::{self, Write};

static SEP: &'static str = ".";

/// Python token specialization.
#[derive(Debug, Clone)]
pub enum Python<'el> {
    /// An imported module.
    Imported {
        /// Module of the imported name.
        module: Cow<'el, str>,
        /// Alias of module.
        alias: Option<Cow<'el, str>>,
        /// Name imported.
        ///
        /// If `None`, last component of module will be used.
        name: Option<Cow<'el, str>>,
    },
}

into_tokens_impl_from!(Python<'el>, Python<'el>);
into_tokens_impl_from!(&'el Python<'el>, Python<'el>);

impl<'el> Python<'el> {
    fn imports<'a>(tokens: &'a Tokens<'a, Self>) -> Option<Tokens<'a, Self>> {
        use self::Python::*;

        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            match *custom {
                Imported {
                    ref module,
                    ref alias,
                    ..
                } => modules.insert((module.as_ref(), alias.as_ref().map(AsRef::as_ref))),
            };
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
    pub fn alias<A: Into<Cow<'el, str>>>(self, new_alias: A) -> Python<'el> {
        use self::Python::*;

        match self {
            Imported { module, name, .. } => Python::Imported {
                module: module,
                alias: Some(new_alias.into()),
                name: name,
            },
        }
    }

    /// Set name for python element.
    pub fn name<A: Into<Cow<'el, str>>>(self, new_name: A) -> Python<'el> {
        use self::Python::*;

        match self {
            Imported { module, alias, .. } => Python::Imported {
                module: module,
                alias: alias,
                name: Some(new_name.into()),
            },
        }
    }
}

impl<'el> Custom for Python<'el> {
    type Extra = ();

    fn format(&self, out: &mut Formatter, _extra: &mut Self::Extra, _level: usize) -> fmt::Result {
        use self::Python::*;

        match *self {
            Imported {
                ref module,
                ref alias,
                ref name,
                ..
            } => {
                let has_module = match *alias {
                    Some(ref alias) => {
                        out.write_str(alias)?;
                        true
                    }
                    None => {
                        if let Some(part) = module.split(SEP).last() {
                            out.write_str(part)?;
                            true
                        } else {
                            false
                        }
                    }
                };

                if let Some(ref name) = *name {
                    if has_module {
                        out.write_str(SEP)?;
                    }

                    out.write_str(name)?;
                }
            }
        }

        Ok(())
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
pub fn imported<'a, M>(module: M) -> Python<'a>
where
    M: Into<Cow<'a, str>>,
{
    Python::Imported {
        module: module.into(),
        alias: None,
        name: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{imported, Python};
    use quoted::Quoted;
    use tokens::Tokens;

    #[test]
    fn test_string() {
        let mut toks: Tokens<Python> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }

    #[test]
    fn test_imported() {
        let mut toks: Tokens<Python> = Tokens::new();
        toks.push(toks![
            imported("collections").name("named_tuple".to_string())
        ]);
        toks.push(toks![imported("collections")]);
        toks.push(toks![
            imported("collections").alias("c").name("named_tuple")
        ]);
        toks.push(toks![imported("collections").alias("c")]);

        assert_eq!(
            Ok("import collections\nimport collections as c\n\ncollections.named_tuple\ncollections\nc.named_tuple\nc\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
