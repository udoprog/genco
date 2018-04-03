//! Specialization for JavaScript code generation.

use super::custom::Custom;
use super::formatter::Formatter;
use super::into_tokens::IntoTokens;
use super::quoted::Quoted;
use super::tokens::Tokens;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt::{self, Write};

static SEP: &'static str = ".";
static PATH_SEP: &'static str = "/";

/// JavaScript token specialization.
#[derive(Debug, Clone)]
pub enum JavaScript<'el> {
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

into_tokens_impl_from!(JavaScript<'el>, JavaScript<'el>);
into_tokens_impl_from!(&'el JavaScript<'el>, JavaScript<'el>);

impl<'el> JavaScript<'el> {
    fn module_to_path(path: &str) -> String {
        let parts: Vec<&str> = path.split(SEP).collect();
        format!("{}.js", parts.join(PATH_SEP))
    }

    fn imports<'a>(tokens: &'a Tokens<'a, Self>) -> Option<Tokens<'a, Self>> {
        use self::JavaScript::*;

        let mut wildcard = BTreeSet::new();

        for custom in tokens.walk_custom() {
            match *custom {
                ImportedAlias {
                    ref module,
                    ref alias,
                    ..
                } => wildcard.insert((module.as_ref(), alias.as_ref())),
            };
        }

        let mut out = Tokens::new();

        for (module, alias) in wildcard {
            let mut s = Tokens::new();

            s.append("import * as ");
            s.append(alias);
            s.append(" from ");
            s.append(Self::module_to_path(module).quoted());
            s.append(";");

            out.push(s);
        }

        Some(out)
    }
}

impl<'el> Custom for JavaScript<'el> {
    type Extra = ();

    fn format(&self, out: &mut Formatter, _extra: &mut Self::Extra, _level: usize) -> fmt::Result {
        use self::JavaScript::*;

        match *self {
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

/// Setup an imported alias element.
pub fn imported_alias<'a, M, N, A>(module: M, name: N, alias: A) -> JavaScript<'a>
where
    M: Into<Cow<'a, str>>,
    N: Into<Cow<'a, str>>,
    A: Into<Cow<'a, str>>,
{
    JavaScript::ImportedAlias {
        module: module.into(),
        name: name.into(),
        alias: alias.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{imported_alias, JavaScript};
    use quoted::Quoted;
    use tokens::Tokens;

    #[test]
    fn test_function() {
        let mut file: Tokens<JavaScript> = Tokens::new();

        file.push("function foo(v) {");
        file.nested(toks!("return v + ", ", World".quoted(), ";"));
        file.push("}");

        file.push(toks!("foo(", "Hello".quoted(), ");"));

        assert_eq!(
            Ok(String::from(
                "function foo(v) {\n  return v + \", World\";\n}\nfoo(\"Hello\");",
            )),
            file.to_string()
        );
    }

    #[test]
    fn test_string() {
        let mut toks: Tokens<JavaScript> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!(Ok(String::from("\"hello \\n world\"")), toks.to_string());
    }

    #[test]
    fn test_imported() {
        let dbg = imported_alias("collections", "vec".to_string(), "list");
        let mut toks: Tokens<JavaScript> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("import * as list from \"collections.js\";\n\nlist.vec\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
