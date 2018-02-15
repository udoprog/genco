//! Specialization for Rust code generation.

use super::custom::Custom;
use super::formatter::Formatter;
use std::fmt::{self, Write};
use super::tokens::Tokens;
use std::collections::BTreeSet;
use std::borrow::Cow;
use super::into_tokens::IntoTokens;

static SEP: &'static str = "::";

/// Rust token specialization.
#[derive(Debug, Clone)]
pub enum Rust<'el> {
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

into_tokens_impl_from!(Rust<'el>, Rust<'el>);
into_tokens_impl_from!(&'el Rust<'el>, Rust<'el>);

impl<'el> Rust<'el> {
    fn imports<'a>(tokens: &'a Tokens<'a, Self>) -> Option<Tokens<'a, Self>> {
        use self::Rust::*;

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

impl<'el> Custom for Rust<'el> {
    type Extra = ();

    fn format(&self, out: &mut Formatter, _extra: &mut Self::Extra, _level: usize) -> fmt::Result {
        use self::Rust::*;

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
    M: Into<Cow<'a, str>>,
    N: Into<Cow<'a, str>>,
{
    Rust::Imported {
        module: module.into(),
        name: name.into(),
    }
}

/// Setup an imported alias element.
pub fn imported_alias<'a, M, N, A>(module: M, name: N, alias: A) -> Rust<'a>
where
    M: Into<Cow<'a, str>>,
    N: Into<Cow<'a, str>>,
    A: Into<Cow<'a, str>>,
{
    Rust::ImportedAlias {
        module: module.into(),
        name: name.into(),
        alias: alias.into(),
    }
}

#[cfg(test)]
mod tests {
    use tokens::Tokens;
    use rust::Rust;
    use quoted::Quoted;
    use super::imported;

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
}
