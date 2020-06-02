//! Specialization for Swift code generation.

use crate::{Cons, Formatter, Lang, LangItem, Tokens};
use std::collections::BTreeSet;
use std::fmt::{self, Write};

/// Name of an imported type.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Name {
    /// Module of the imported name.
    module: Option<Cons<'static>>,
    /// Name imported.
    name: Cons<'static>,
}

/// Swift token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Swift(());

/// An imported Swift element.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Imported {
    /// A regular type.
    Type {
        /// The name being referenced.
        name: Name,
    },
    /// A map, [<key>: <value>].
    Map {
        /// Key of the map.
        key: Box<Imported>,
        /// Value of the map.
        value: Box<Imported>,
    },
    /// An array, [<inner>].
    Array {
        /// Inner value of the array.
        inner: Box<Imported>,
    },
}

impl_lang_item!(Imported, Swift);

impl LangItem<Swift> for Imported {
    /// Format the language item appropriately.
    fn format(&self, out: &mut Formatter, config: &mut (), level: usize) -> fmt::Result {
        match self {
            Self::Type {
                name: Name { name, .. },
                ..
            } => {
                out.write_str(name)?;
            }
            Self::Map { key, value, .. } => {
                out.write_str("[")?;
                key.format(out, config, level + 1)?;
                out.write_str(": ")?;
                value.format(out, config, level + 1)?;
                out.write_str("]")?;
            }
            Self::Array { inner, .. } => {
                out.write_str("[")?;
                inner.format(out, config, level + 1)?;
                out.write_str("]")?;
            }
        }

        Ok(())
    }

    /// Coerce into an imported type.
    ///
    /// This is used for import resolution for custom language items.
    fn as_import(&self) -> Option<&Self> {
        Some(self)
    }
}

impl Swift {
    fn type_imports(item: &Imported, modules: &mut BTreeSet<Cons<'static>>) {
        use self::Imported::*;

        match item {
            Type { name, .. } => {
                if let Some(module) = name.module.as_ref() {
                    modules.insert(module.clone());
                }
            }
            Map { key, value, .. } => {
                Self::type_imports(&*key, modules);
                Self::type_imports(&*value, modules);
            }
            Array { inner, .. } => {
                Self::type_imports(&*inner, modules);
            }
        };
    }

    fn imports<'el>(tokens: &Tokens<'el, Self>) -> Option<Tokens<'el, Self>> {
        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            if let Some(import) = custom.as_import() {
                Self::type_imports(import, &mut modules);
            }
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

impl Lang for Swift {
    type Config = ();
    type Import = Imported;

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
        tokens: Tokens<'_, Self>,
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
pub fn imported<M, N>(module: M, name: N) -> Imported
where
    M: Into<Cons<'static>>,
    N: Into<Cons<'static>>,
{
    Imported::Type {
        name: Name {
            module: Some(module.into()),
            name: name.into(),
        },
    }
}

/// Setup a local element.
pub fn local<N>(name: N) -> Imported
where
    N: Into<Cons<'static>>,
{
    Imported::Type {
        name: Name {
            module: None,
            name: name.into(),
        },
    }
}

/// Setup a map.
pub fn map<K, V>(key: K, value: V) -> Imported
where
    K: Into<Imported>,
    V: Into<Imported>,
{
    Imported::Map {
        key: Box::new(key.into()),
        value: Box::new(value.into()),
    }
}

/// Setup an array.
pub fn array<'a, I>(inner: I) -> Imported
where
    I: Into<Imported>,
{
    Imported::Array {
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
