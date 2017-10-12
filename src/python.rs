//! Specialization for Python code generation.

use super::custom::Custom;
use super::formatter::Formatter;
use std::fmt::{self, Write};
use std::borrow::Cow;
use super::tokens::Tokens;
use std::collections::BTreeSet;

static SEP: &'static str = ".";

/// Python token specialization.
#[derive(Debug, Clone)]
pub enum Python<'el> {
    /// An imported name.
    Imported {
        /// Module of the imported name.
        module: Cow<'el, str>,
        /// Name imported.
        name: Cow<'el, str>,
    },
    /// An imported module as an alias.
    ImportedAlias {
        /// Module of the imported name.
        module: Cow<'el, str>,
        /// Name imported.
        name: Cow<'el, str>,
        /// Alias of module.
        alias: Cow<'el, str>,
    },
}

impl<'el> Python<'el> {
    fn imports<'a>(tokens: &'a Tokens<'a, Self>) -> Option<Tokens<'a, Self>> {
        use self::Python::*;

        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            match *custom {
                Imported { ref module, .. } => modules.insert((module.as_ref(), None)),
                ImportedAlias {
                    ref module,
                    ref alias,
                    ..
                } => modules.insert((module.as_ref(), Some(alias.as_ref()))),
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
}

impl<'el> Custom for Python<'el> {
    type Extra = ();

    fn format(&self, out: &mut Formatter, _extra: &mut Self::Extra, _level: usize) -> fmt::Result {
        use self::Python::*;

        match *self {
            Imported {
                ref module,
                ref name,
                ..
            } => {
                if let Some(part) = module.split(SEP).last() {
                    out.write_str(part)?;
                    out.write_str(SEP)?;
                }

                out.write_str(name)?;
            }
            ImportedAlias {
                ref alias,
                ref name,
                ..
            } => {
                out.write_str(alias)?;
                out.write_str(SEP)?;
                out.write_str(name)?;
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
pub fn imported<'a>(module: Cow<'a, str>, name: Cow<'a, str>) -> Python<'a> {
    Python::Imported {
        module: module,
        name: name,
    }
}

/// Setup an imported element from borrowed components.
pub fn imported_ref<'a>(module: &'a str, name: &'a str) -> Python<'a> {
    Python::Imported {
        module: Cow::Borrowed(module),
        name: Cow::Borrowed(name),
    }
}

/// Setup an imported alias element.
pub fn imported_alias<'a>(
    module: Cow<'a, str>,
    name: Cow<'a, str>,
    alias: Cow<'a, str>,
) -> Python<'a> {
    Python::ImportedAlias {
        module: module,
        name: name,
        alias: alias,
    }
}

/// Setup an imported alias element from borrowed components.
pub fn imported_alias_ref<'a>(module: &'a str, name: &'a str, alias: &'a str) -> Python<'a> {
    imported_alias(
        Cow::Borrowed(module),
        Cow::Borrowed(name),
        Cow::Borrowed(alias),
    )
}

#[cfg(test)]
mod tests {
    use tokens::Tokens;
    use super::{Python, imported_ref};
    use quoted::Quoted;

    #[test]
    fn test_string() {
        let mut toks: Tokens<Python> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }

    #[test]
    fn test_imported() {
        let dbg = imported_ref("collections", "named_tuple");
        let mut toks: Tokens<Python> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("import collections\n\ncollections.named_tuple\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
