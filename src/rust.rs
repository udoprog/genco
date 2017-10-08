//! Specialization for Rust code generation.

use super::custom::Custom;
use super::formatter::Formatter;
use std::fmt;
use super::tokens::Tokens;
use std::collections::BTreeSet;
use std::borrow::Cow;

static SEP: &'static str = "::";

/// Rust token specialization.
#[derive(Debug, Clone)]
pub enum Rust<'element> {
    /// An imported name.
    Imported {
        /// Module of the imported name.
        module: Cow<'element, str>,
        /// Name imported.
        name: Cow<'element, str>,
    },
    /// An imported module as an alias.
    ImportedAlias {
        /// Module of the imported name.
        module: Cow<'element, str>,
        /// Name imported.
        name: Cow<'element, str>,
        /// Alias of module.
        alias: Cow<'element, str>,
    },
}

impl<'element> Rust<'element> {
    fn imports<'a>(tokens: &'a Tokens<'a, Self>) -> Option<Tokens<'a, Self>> {
        use self::Rust::*;

        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            match *custom {
                Imported {
                    ref module,
                    ..
                } => modules.insert((module.as_ref(), None)),
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

            s.append("use ");
            s.append(module);

            if let Some(alias) = alias {
                s.append(" as ");
                s.append(alias);
            }

            s.append(";");

            out.push(s);
        }

        Some(out)
    }
}

impl<'element> Custom for Rust<'element> {
    type Extra = ();

    fn format(&self, out: &mut Formatter, _extra: &mut Self::Extra, _level: usize) -> fmt::Result {
        use self::Rust::*;

        match *self {
            Imported { ref module, ref name, .. } => {
                if let Some(part) = module.split(SEP).last() {
                    out.write_str(part)?;
                    out.write_str(SEP)?;
                }

                out.write_str(name)?;
            }
            ImportedAlias { ref alias, ref name, .. } => {
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
pub fn imported<'a>(module: Cow<'a, str>, name: Cow<'a, str>) -> Rust<'a> {
    Rust::Imported {
        module: module,
        name: name,
    }
}

/// Setup an imported element from borrowed components.
pub fn imported_ref<'a>(module: &'a str, name: &'a str) -> Rust<'a> {
    Rust::Imported {
        module: Cow::Borrowed(module),
        name: Cow::Borrowed(name),
    }
}

/// Setup an imported alias element.
pub fn imported_alias<'a>(
    module: Cow<'a, str>,
    name: Cow<'a, str>,
    alias: Cow<'a, str>,
) -> Rust<'a> {
    Rust::ImportedAlias {
        module: module,
        name: name,
        alias: alias,
    }
}

/// Setup an imported alias element from borrowed components.
pub fn imported_alias_ref<'a>(module: &'a str, name: &'a str, alias: &'a str) -> Rust<'a> {
    imported_alias(
        Cow::Borrowed(module),
        Cow::Borrowed(name),
        Cow::Borrowed(alias),
    )
}

#[cfg(test)]
mod tests {
    use tokens::Tokens;
    use rust::Rust;
    use quoted::Quoted;
    use super::imported_ref;

    #[test]
    fn test_string() {
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.append("hello \n world".quoted());
        let res = toks.to_string();

        assert_eq!(Ok("\"hello \\n world\""), res.as_ref().map(|s| s.as_str()));
    }

    #[test]
    fn test_imported() {
        let dbg = imported_ref("std::fmt", "Debug");
        let mut toks: Tokens<Rust> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("use std::fmt;\n\nfmt::Debug"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
