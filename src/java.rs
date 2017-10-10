//! Specialization for Java code generation.

use super::custom::Custom;
use super::formatter::Formatter;
use std::fmt;
use std::borrow::Cow;
use super::tokens::Tokens;
use std::collections::{HashMap, BTreeSet};

static JAVA_LANG: &'static str = "java.lang";
static SEP: &'static str = ".";

/// Java token specialization.
#[derive(Debug, Clone)]
pub enum Java<'el> {
    /// An imported name.
    Imported {
        /// Package of the imported name.
        package: Cow<'el, str>,
        /// Name imported.
        name: Cow<'el, str>,
    },
}

/// Extra data for Java formatting.
#[derive(Debug, Default)]
pub struct JavaExtra {
    /// Types which has been imported into the local namespace.
    imported: HashMap<String, String>,
}

impl<'el> Java<'el> {
    fn imports<'a>(
        tokens: &'a Tokens<'a, Self>,
        extra: &mut JavaExtra,
    ) -> Option<Tokens<'a, Self>> {
        use self::Java::*;

        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            match *custom {
                Imported {
                    ref package,
                    ref name,
                } => modules.insert((package.as_ref(), name.as_ref())),
            };
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for (package, name) in modules {
            if extra.imported.contains_key(name) {
                continue;
            }

            if package == JAVA_LANG {
                continue;
            }

            out.push(toks!("import ", package, SEP, name, ";"));
            extra.imported.insert(name.to_string(), package.to_string());
        }

        Some(out)
    }
}

impl<'el> Custom for Java<'el> {
    type Extra = JavaExtra;

    fn format(&self, out: &mut Formatter, extra: &mut Self::Extra, _level: usize) -> fmt::Result {
        use self::Java::*;

        match *self {
            Imported {
                ref package,
                ref name,
                ..
            } => {
                if package != JAVA_LANG &&
                    extra.imported.get(name.as_ref()).map(String::as_str) !=
                        Some(package.as_ref())
                {
                    out.write_str(package.as_ref())?;
                    out.write_str(SEP)?;
                }

                out.write_str(name.as_ref())?;
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
            }
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

        if let Some(imports) = Self::imports(&tokens, extra) {
            toks.push(imports);
        }

        toks.push_ref(&tokens);
        toks.join_line_spacing().format(out, extra, level)
    }
}

/// Setup an imported element.
pub fn imported<'a>(package: Cow<'a, str>, name: Cow<'a, str>) -> Java<'a> {
    Java::Imported {
        package: package,
        name: name,
    }
}

/// Setup an imported element from borrowed components.
pub fn imported_ref<'a>(package: &'a str, name: &'a str) -> Java<'a> {
    Java::Imported {
        package: Cow::Borrowed(package),
        name: Cow::Borrowed(name),
    }
}

#[cfg(test)]
mod tests {
    use tokens::Tokens;
    use java::Java;
    use quoted::Quoted;
    use super::imported_ref;

    #[test]
    fn test_string() {
        let mut toks: Tokens<Java> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }

    #[test]
    fn test_imported() {
        let integer = imported_ref("java.lang", "Integer");
        let a = imported_ref("java.io", "A");
        let b = imported_ref("java.io", "B");
        let ob = imported_ref("java.util", "B");

        let toks = toks!(integer, a, b, ob).join_spacing();

        assert_eq!(
            Ok(
                "import java.io.A;\nimport java.io.B;\n\nInteger A B java.util.B\n",
            ),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
