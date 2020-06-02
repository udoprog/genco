//! Specialization for JavaScript code generation.

use crate::{Cons, Formatter, Lang, Quoted, Tokens};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write};

static SEP: &'static str = ".";
static PATH_SEP: &'static str = "/";

/// JavaScript token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct JavaScript<'el> {
    /// Module of the imported name.
    module: Option<Cons<'el>>,
    /// Name imported.
    name: Cons<'el>,
    /// Alias of module.
    alias: Option<Cons<'el>>,
}

impl<'el> fmt::Display for JavaScript<'el> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref alias) = self.alias {
            fmt.write_str(alias.as_ref())?;
            fmt.write_str(SEP)?;
        }

        fmt.write_str(self.name.as_ref())?;
        Ok(())
    }
}

impl<'el> JavaScript<'el> {
    /// Alias the given type.
    pub fn alias<N: Into<Cons<'el>>>(self, alias: N) -> JavaScript<'el> {
        JavaScript {
            alias: Some(alias.into()),
            ..self
        }
    }

    fn module_to_path(path: &str) -> String {
        let parts: Vec<&str> = path.split(SEP).collect();
        format!("{}.js", parts.join(PATH_SEP))
    }

    fn imports(tokens: &Tokens<'el, Self>) -> Option<Tokens<'el, Self>> {
        let mut sets = BTreeMap::new();
        let mut wildcard = BTreeSet::new();

        for custom in tokens.walk_custom() {
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

impl<'el> Lang<'el> for JavaScript<'el> {
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
pub fn imported<'el, M, N>(module: M, name: N) -> JavaScript<'el>
where
    M: Into<Cons<'el>>,
    N: Into<Cons<'el>>,
{
    JavaScript {
        module: Some(module.into()),
        name: name.into(),
        alias: None,
    }
}

/// Setup a local element.
pub fn local<'el, N>(name: N) -> JavaScript<'el>
where
    N: Into<Cons<'el>>,
{
    JavaScript {
        module: None,
        name: name.into(),
        alias: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{imported, local, JavaScript};
    use crate::quoted::Quoted;
    use crate::tokens::Tokens;

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
        let mut toks: Tokens<JavaScript> = Tokens::new();
        toks.push(toks!(imported("collections", "vec").alias("list")));
        toks.push(toks!(imported("collections", "vec")));

        assert_eq!(
            Ok("import {vec} from \"collections.js\";\nimport * as list from \"collections.js\";\n\nlist.vec\nvec\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }

    #[test]
    fn test_local() {
        let dbg = local("vec");
        let mut toks: Tokens<JavaScript> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(Ok("vec\n"), toks.to_file().as_ref().map(|s| s.as_str()));
    }
}
