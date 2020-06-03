//! Specialization for Go code generation.

use crate::{Cons, Formatter, Lang, LangItem, Quoted};
use std::collections::BTreeSet;
use std::fmt::{self, Write};

/// Tokens container specialization for Go.
pub type Tokens<'el> = crate::Tokens<'el, Go>;

impl_lang_item!(Imported, Go);

const SEP: &str = ".";

/// Name of an imported type.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Name {
    /// Module of the imported name.
    module: Option<Cons<'static>>,
    /// Name imported.
    name: Cons<'static>,
}

/// Go token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Imported {
    /// A regular type.
    Type {
        /// The name being referenced.
        name: Name,
    },
    /// A map, map[<key>]<value>.
    Map {
        /// Key of the map.
        key: Box<Imported>,
        /// Value of the map.
        value: Box<Imported>,
    },
    /// An array, []<inner>.
    Array {
        /// Inner value of the array.
        inner: Box<Imported>,
    },
    /// An interface type, interface{}.
    Interface,
}

impl Imported {
    fn type_imports(&self, modules: &mut BTreeSet<Cons<'static>>) {
        match self {
            Self::Type { name, .. } => {
                if let Some(module) = name.module.clone() {
                    modules.insert(module);
                }
            }
            Self::Map { key, value, .. } => {
                key.type_imports(modules);
                value.type_imports(modules);
            }
            Self::Array { inner, .. } => {
                inner.type_imports(modules);
            }
            Self::Interface => {}
        };
    }
}

impl LangItem<Go> for Imported {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        match self {
            Self::Type {
                name: Name { module, name, .. },
                ..
            } => {
                if let Some(module) = module.as_ref().and_then(|m| m.as_ref().split("/").last()) {
                    out.write_str(module)?;
                    out.write_str(SEP)?;
                }

                out.write_str(name)?;
            }
            Self::Map { key, value, .. } => {
                out.write_str("map[")?;
                key.format(out, config, level + 1)?;
                out.write_str("]")?;
                value.format(out, config, level + 1)?;
            }
            Self::Array { inner, .. } => {
                out.write_str("[")?;
                out.write_str("]")?;
                inner.format(out, config, level + 1)?;
            }
            Self::Interface => {
                out.write_str("interface{}")?;
            }
        }

        Ok(())
    }

    fn as_import(&self) -> Option<&Self> {
        Some(self)
    }
}

/// Config data for Go.
#[derive(Debug)]
pub struct Config {
    package: String,
}

impl Config {
    /// Build the config structure from a package.
    pub fn from_package<S: AsRef<str>>(package: S) -> Self {
        Self {
            package: package.as_ref().to_string(),
        }
    }
}

/// Language specialization for Go.
pub struct Go(());

impl Go {
    fn imports<'el>(tokens: &Tokens<'el>) -> Option<Tokens<'el>> {
        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            if let Some(import) = custom.as_import() {
                import.type_imports(&mut modules);
            }
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

impl Lang for Go {
    type Config = Config;
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
        tokens: Tokens<'_>,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks = Tokens::new();

        toks.append("package");
        toks.spacing();
        toks.append(config.package.clone());
        toks.line_spacing();

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
pub fn array<I>(inner: I) -> Imported
where
    I: Into<Imported>,
{
    Imported::Array {
        inner: Box::new(inner.into()),
    }
}

/// Setup an interface.
pub fn interface<'a>() -> Imported {
    Imported::Interface
}

#[cfg(test)]
mod tests {
    use super::{array, imported, interface, map, Config, Go, Tokens};
    use crate as genco;
    use crate::{quote, FormatterConfig, Quoted};

    #[test]
    fn test_string() {
        let mut toks = Tokens::new();
        toks.append("hello \n world".quoted());
        let res = toks.to_string_with(
            Config::from_package("foo"),
            FormatterConfig::from_lang::<Go>(),
        );

        assert_eq!(Ok("\"hello \\n world\""), res.as_ref().map(|s| s.as_str()));
    }

    #[test]
    fn test_imported() {
        let dbg = imported("foo", "Debug");
        let mut toks = Tokens::new();
        toks.push(quote!(#dbg));

        assert_eq!(
            vec!["package foo", "", "import \"foo\"", "", "foo.Debug", ""],
            toks.to_file_vec_with(
                Config::from_package("foo"),
                FormatterConfig::from_lang::<Go>()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_map() {
        let keyed = map(imported("foo", "Debug"), interface());

        let mut toks = Tokens::new();
        toks.push(quote!(#keyed));

        assert_eq!(
            vec![
                "package foo",
                "",
                "import \"foo\"",
                "",
                "map[foo.Debug]interface{}",
                ""
            ],
            toks.to_file_vec_with(
                Config::from_package("foo"),
                FormatterConfig::from_lang::<Go>()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_array() {
        let keyed = array(imported("foo", "Debug"));

        let mut toks = Tokens::new();
        toks.push(quote!(#keyed));

        assert_eq!(
            Ok("package foo\n\nimport \"foo\"\n\n[]foo.Debug\n"),
            toks.to_file_string_with(
                Config::from_package("foo"),
                FormatterConfig::from_lang::<Go>()
            )
            .as_ref()
            .map(|s| s.as_str())
        );
    }
}
