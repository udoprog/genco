//! Specialization for Swift code generation.

use crate::{Cons, Custom, Formatter, Tokens};
use std::collections::BTreeSet;
use std::fmt::{self, Write};

/// Name of an imported type.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Name<'el> {
    /// Module of the imported name.
    module: Option<Cons<'el>>,
    /// Name imported.
    name: Cons<'el>,
}

/// Swift token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Swift<'el> {
    /// A regular type.
    Type {
        /// The name being referenced.
        name: Name<'el>,
    },
    /// A map, [<key>: <value>].
    Map {
        /// Key of the map.
        key: Box<Swift<'el>>,
        /// Value of the map.
        value: Box<Swift<'el>>,
    },
    /// An array, [<inner>].
    Array {
        /// Inner value of the array.
        inner: Box<Swift<'el>>,
    },
}

impl<'el> Swift<'el> {
    fn type_imports(swift: &Swift<'el>, modules: &mut BTreeSet<Cons<'el>>) {
        use self::Swift::*;

        match *swift {
            Type { ref name, .. } => {
                if let Some(module) = name.module.as_ref() {
                    modules.insert(module.clone());
                }
            }
            Map {
                ref key, ref value, ..
            } => {
                Self::type_imports(key, modules);
                Self::type_imports(value, modules);
            }
            Array { ref inner, .. } => {
                Self::type_imports(inner, modules);
            }
        };
    }

    fn imports(tokens: &Tokens<'el, Self>) -> Option<Tokens<'el, Self>> {
        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            Self::type_imports(custom, &mut modules);
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for module in modules {
            let mut s = Tokens::new();

            s.append("import ");
            s.append(module);

            out.push(s);
        }

        Some(out)
    }
}

impl<'el> Custom<'el> for Swift<'el> {
    type Config = ();

    fn format(&self, out: &mut Formatter, config: &mut Self::Config, level: usize) -> fmt::Result {
        use self::Swift::*;

        match *self {
            Type {
                name: Name { ref name, .. },
                ..
            } => {
                out.write_str(name)?;
            }
            Map {
                ref key, ref value, ..
            } => {
                out.write_str("[")?;
                key.format(out, config, level + 1)?;
                out.write_str(": ")?;
                value.format(out, config, level + 1)?;
                out.write_str("]")?;
            }
            Array { ref inner, .. } => {
                out.write_str("[")?;
                inner.format(out, config, level + 1)?;
                out.write_str("]")?;
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

        toks.extend(tokens);
        toks.format(out, config, level)
    }
}

/// Setup an imported element.
pub fn imported<'a, M, N>(module: M, name: N) -> Swift<'a>
where
    M: Into<Cons<'a>>,
    N: Into<Cons<'a>>,
{
    Swift::Type {
        name: Name {
            module: Some(module.into()),
            name: name.into(),
        },
    }
}

/// Setup a local element.
pub fn local<'a, N>(name: N) -> Swift<'a>
where
    N: Into<Cons<'a>>,
{
    Swift::Type {
        name: Name {
            module: None,
            name: name.into(),
        },
    }
}

/// Setup a map.
pub fn map<'a, K, V>(key: K, value: V) -> Swift<'a>
where
    K: Into<Swift<'a>>,
    V: Into<Swift<'a>>,
{
    Swift::Map {
        key: Box::new(key.into()),
        value: Box::new(value.into()),
    }
}

/// Setup an array.
pub fn array<'a, I>(inner: I) -> Swift<'a>
where
    I: Into<Swift<'a>>,
{
    Swift::Array {
        inner: Box::new(inner.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{array, imported, local, map, Swift};
    use crate::{Quoted, Tokens};

    #[test]
    fn test_string() {
        let mut toks: Tokens<Swift> = Tokens::new();
        toks.append("hello \n world".quoted());
        let res = toks.to_string();

        assert_eq!(Ok("\"hello \\n world\""), res.as_ref().map(|s| s.as_str()));
    }

    #[test]
    fn test_imported() {
        let dbg = imported("Foo", "Debug");
        let mut toks: Tokens<Swift> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("import Foo\n\nDebug\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }

    #[test]
    fn test_array() {
        let dbg = array(imported("Foo", "Debug"));
        let mut toks: Tokens<Swift> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("import Foo\n\n[Debug]\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }

    #[test]
    fn test_map() {
        let dbg = map(local("String"), imported("Foo", "Debug"));
        let mut toks: Tokens<Swift> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("import Foo\n\n[String: Debug]\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
