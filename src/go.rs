//! Specialization for Go code generation.

use {Custom, Formatter, Quoted, Tokens};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt::{self, Write};

const SEP: &str = ".";

/// Go token specialization.
#[derive(Debug, Clone)]
pub enum Go<'el> {
    /// An imported name.
    Imported {
        /// Module of the imported name.
        module: Cow<'el, str>,
        /// Name imported.
        name: Cow<'el, str>,
    },
    /// An local name (same package)
    Local {
        /// Name.
        name: Cow<'el, str>,
    },
}

impl<'el> Go<'el> {
    fn imports<'a>(tokens: &'a Tokens<'a, Self>) -> Option<Tokens<'a, Self>> {
        use self::Go::*;

        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            match *custom {
                Imported { ref module, .. } => {
                    modules.insert(module);
                }
                _ => {}
            };
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for module in modules {
            let mut s = Tokens::new();

            s.append("import ");
            s.append(module.quoted());

            out.push(s);
        }

        Some(out)
    }
}

/// Extra data for Go.
#[derive(Debug)]
pub struct Extra {
    package: String,
}

impl Extra {
    /// Build the extra structure from a package.
    pub fn from_package<S: AsRef<str>>(package: S) -> Self {
        Self {
            package: package.as_ref().to_string(),
        }
    }
}

impl<'el> Custom for Go<'el> {
    type Extra = Extra;

    fn format(&self, out: &mut Formatter, _extra: &mut Self::Extra, _level: usize) -> fmt::Result {
        use self::Go::*;

        match *self {
            Imported {
                ref module,
                ref name,
                ..
            } => {
                if let Some(module) = module.split("/").last() {
                    out.write_str(module)?;
                    out.write_str(SEP)?;
                }

                out.write_str(name)?;
            }
            Local { ref name } => {
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

        toks.push_into(|t| {
            t.append("package ");
            t.append(extra.package.to_string());
        });

        if let Some(imports) = Self::imports(&tokens) {
            toks.push(imports);
        }

        toks.push_ref(&tokens);
        toks.join_line_spacing().format(out, extra, level)
    }
}

/// Setup an imported element.
pub fn imported<'a, M, N>(module: M, name: N) -> Go<'a>
where
    M: Into<Cow<'a, str>>,
    N: Into<Cow<'a, str>>,
{
    Go::Imported {
        module: module.into(),
        name: name.into(),
    }
}

/// Setup a local element.
pub fn local<'a, N>(name: N) -> Go<'a>
where
    N: Into<Cow<'a, str>>,
{
    Go::Local { name: name.into() }
}

#[cfg(test)]
mod tests {
    use super::{imported, Extra, Go};
    use genco::{Quoted, Tokens};

    #[test]
    fn test_string() {
        let mut toks: Tokens<Go> = Tokens::new();
        toks.append("hello \n world".quoted());
        let res = toks.to_string_with(Extra::from_package("foo"));

        assert_eq!(Ok("\"hello \\n world\""), res.as_ref().map(|s| s.as_str()));
    }

    #[test]
    fn test_imported() {
        let dbg = imported("foo", "Debug");
        let mut toks: Tokens<Go> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("package foo\n\nimport \"foo\"\n\nfoo.Debug\n"),
            toks.to_file_with(Extra::from_package("foo"))
                .as_ref()
                .map(|s| s.as_str())
        );
    }
}
