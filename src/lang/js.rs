//! Specialization for JavaScript code generation.

use crate::{Cons, Formatter, Lang, LangItem, Quoted};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write};

/// Tokens container specialization for Rust.
pub type Tokens<'el> = crate::Tokens<'el, JavaScript>;

impl_lang_item!(Imported, JavaScript);

static SEP: &'static str = ".";
static PATH_SEP: &'static str = "/";

/// An imported item in JavaScript.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Imported {
    /// Module of the imported name.
    module: Option<Cons<'static>>,
    /// Name imported.
    name: Cons<'static>,
    /// Alias of module.
    alias: Option<Cons<'static>>,
}

impl Imported {
    /// Alias the given type.
    pub fn alias<N: Into<Cons<'static>>>(self, alias: N) -> Self {
        Self {
            alias: Some(alias.into()),
            ..self
        }
    }
}

impl fmt::Display for Imported {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(alias) = &self.alias {
            fmt.write_str(alias)?;
            fmt.write_str(SEP)?;
        }

        fmt.write_str(self.name.as_ref())?;
        Ok(())
    }
}

impl LangItem<JavaScript> for Imported {
    fn format(&self, out: &mut Formatter, _: &mut (), _: usize) -> fmt::Result {
        write!(out, "{}", self)
    }

    fn as_import(&self) -> Option<&Self> {
        Some(self)
    }
}

/// JavaScript language specialization.
pub struct JavaScript(());

impl JavaScript {
    /// Convert a module into a path.
    fn module_to_path(path: &str) -> String {
        let parts: Vec<&str> = path.split(SEP).collect();
        format!("{}.js", parts.join(PATH_SEP))
    }

    /// Translate imports into the necessary tokens.
    fn imports<'el>(tokens: &Tokens<'el>) -> Option<Tokens<'el>> {
        let mut sets = BTreeMap::new();
        let mut wildcard = BTreeSet::new();

        for custom in tokens.walk_custom() {
            if let Some(custom) = custom.as_import() {
                match (&custom.module, &custom.alias) {
                    (&Some(ref module), &None) => {
                        sets.entry(module.clone())
                            .or_insert_with(Tokens::new)
                            .append(custom.name.clone());
                    }
                    (&Some(ref module), &Some(ref alias)) => {
                        wildcard.insert((module.clone(), alias.clone()));
                    }
                    _ => {}
                }
            }
        }

        if wildcard.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for (module, names) in sets {
            let mut s = Tokens::new();

            s.append("import {");

            let mut it = names.into_iter();

            if let Some(name) = it.next() {
                s.append(name);
            }

            for name in it {
                s.append(", ");
                s.append(name);
            }

            s.append("} from ");
            s.append(Self::module_to_path(&*module).quoted());
            s.append(";");

            out.push(s);
        }

        for (module, alias) in wildcard {
            let mut s = Tokens::new();

            s.append("import * as ");
            s.append(alias);
            s.append(" from ");
            s.append(Self::module_to_path(&*module).quoted());
            s.append(";");

            out.push(s);
        }

        Some(out)
    }
}

impl Lang for JavaScript {
    type Config = ();
    type Import = Imported;

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

        toks.append(tokens);
        toks.format(out, config, level)
    }
}

/// Setup an imported element.
pub fn imported<'el, M, N>(module: M, name: N) -> Imported
where
    M: Into<Cons<'static>>,
    N: Into<Cons<'static>>,
{
    Imported {
        module: Some(module.into()),
        name: name.into(),
        alias: None,
    }
}

/// Setup a local element.
pub fn local<'el, N>(name: N) -> Imported
where
    N: Into<Cons<'static>>,
{
    Imported {
        module: None,
        name: name.into(),
        alias: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{imported, local, Tokens};
    use crate::Quoted;

    #[test]
    fn test_function() {
        let mut file = Tokens::new();

        file.push("function foo(v) {");
        file.nested(toks!("return v + ", ", World".quoted(), ";"));
        file.push("}");

        file.push(toks!("foo(", "Hello".quoted(), ");"));

        assert_eq!(
            "function foo(v) {\n    return v + \", World\";\n}\nfoo(\"Hello\");",
            file.to_string().unwrap()
        );
    }

    #[test]
    fn test_string() {
        let mut toks = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!(Ok(String::from("\"hello \\n world\"")), toks.to_string());
    }

    #[test]
    fn test_imported() {
        let mut toks = Tokens::new();
        toks.push(toks!(imported("collections", "vec").alias("list")));
        toks.push(toks!(imported("collections", "vec")));

        assert_eq!(
            Ok("import {vec} from \"collections.js\";\nimport * as list from \"collections.js\";\n\nlist.vec\nvec\n"),
            toks.to_file_string().as_ref().map(|s| s.as_str())
        );
    }

    #[test]
    fn test_local() {
        let dbg = local("vec");
        let mut toks = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("vec\n"),
            toks.to_file_string().as_ref().map(|s| s.as_str())
        );
    }
}
