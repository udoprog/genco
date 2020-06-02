//! Specialization for Go code generation.

use crate::{Cons, Custom, Formatter, Quoted};
use std::collections::BTreeSet;
use std::fmt::{self, Write};

/// Tokens container specialization for Go.
pub type Tokens<'el> = crate::Tokens<'el, Go<'el>>;

const SEP: &str = ".";

/// Name of an imported type.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Name<'el> {
    /// Module of the imported name.
    module: Option<Cons<'el>>,
    /// Name imported.
    name: Cons<'el>,
}

/// Go token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Go<'el> {
    /// A regular type.
    Type {
        /// The name being referenced.
        name: Name<'el>,
    },
    /// A map, map[<key>]<value>.
    Map {
        /// Key of the map.
        key: Box<Go<'el>>,
        /// Value of the map.
        value: Box<Go<'el>>,
    },
    /// An array, []<inner>.
    Array {
        /// Inner value of the array.
        inner: Box<Go<'el>>,
    },
    /// An interface type, interface{}.
    Interface,
}

impl<'el> Go<'el> {
    fn type_imports(go: &Go<'el>, modules: &mut BTreeSet<Cons<'el>>) {
        use self::Go::*;

        match *go {
            Type { ref name, .. } => {
                if let Some(module) = name.module.clone() {
                    modules.insert(module);
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
            Interface => {}
        };
    }

    fn imports(tokens: &Tokens<'el>) -> Option<Tokens<'el>> {
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
            s.append(module.quoted());

            out.push(s);
        }

        Some(out)
    }
}

/// Config data for Go.
#[derive(Debug)]
pub struct Config {
    package: String,
}

impl crate::Config for Config {
    fn indentation(&mut self) -> usize {
        4
    }
}

impl Config {
    /// Build the config structure from a package.
    pub fn from_package<S: AsRef<str>>(package: S) -> Self {
        Self {
            package: package.as_ref().to_string(),
        }
    }
}

impl<'el> Custom<'el> for Go<'el> {
    type Config = Config;

    fn format(&self, out: &mut Formatter, config: &mut Self::Config, level: usize) -> fmt::Result {
        use self::Go::*;

        match *self {
            Type {
                name:
                    Name {
                        ref module,
                        ref name,
                        ..
                    },
                ..
            } => {
                if let Some(module) = module.as_ref().and_then(|m| m.as_ref().split("/").last()) {
                    out.write_str(module)?;
                    out.write_str(SEP)?;
                }

                out.write_str(name)?;
            }
            Map {
                ref key, ref value, ..
            } => {
                out.write_str("map[")?;
                key.format(out, config, level + 1)?;
                out.write_str("]")?;
                value.format(out, config, level + 1)?;
            }
            Array { ref inner, .. } => {
                out.write_str("[")?;
                out.write_str("]")?;
                inner.format(out, config, level + 1)?;
            }
            Interface => {
                out.write_str("interface{}")?;
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
        tokens: Tokens<'el>,
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
pub fn imported<'a, M, N>(module: M, name: N) -> Go<'a>
where
    M: Into<Cons<'a>>,
    N: Into<Cons<'a>>,
{
    Go::Type {
        name: Name {
            module: Some(module.into()),
            name: name.into(),
        },
    }
}

/// Setup a local element.
pub fn local<'a, N>(name: N) -> Go<'a>
where
    N: Into<Cons<'a>>,
{
    Go::Type {
        name: Name {
            module: None,
            name: name.into(),
        },
    }
}

/// Setup a map.
pub fn map<'a, K, V>(key: K, value: V) -> Go<'a>
where
    K: Into<Go<'a>>,
    V: Into<Go<'a>>,
{
    Go::Map {
        key: Box::new(key.into()),
        value: Box::new(value.into()),
    }
}

/// Setup an array.
pub fn array<'a, I>(inner: I) -> Go<'a>
where
    I: Into<Go<'a>>,
{
    Go::Array {
        inner: Box::new(inner.into()),
    }
}

/// Setup an interface.
pub fn interface<'a>() -> Go<'a> {
    Go::Interface
}

#[cfg(test)]
mod tests {
    use super::{array, imported, interface, map, Config, Go};
    use crate::{Quoted, Tokens};

    #[test]
    fn test_string() {
        let mut toks: Tokens<Go> = Tokens::new();
        toks.append("hello \n world".quoted());
        let res = toks.to_string_with(Config::from_package("foo"));

        assert_eq!(Ok("\"hello \\n world\""), res.as_ref().map(|s| s.as_str()));
    }

    #[test]
    fn test_imported() {
        let dbg = imported("foo", "Debug");
        let mut toks: Tokens<Go> = Tokens::new();
        toks.push(toks!(&dbg));

        assert_eq!(
            Ok("package foo\n\nimport \"foo\"\n\nfoo.Debug\n"),
            toks.to_file_with(Config::from_package("foo"))
                .as_ref()
                .map(|s| s.as_str())
        );
    }

    #[test]
    fn test_map() {
        let keyed = map(imported("foo", "Debug"), interface());

        let mut toks: Tokens<Go> = Tokens::new();
        toks.push(toks!(&keyed));

        assert_eq!(
            Ok("package foo\n\nimport \"foo\"\n\nmap[foo.Debug]interface{}\n"),
            toks.to_file_with(Config::from_package("foo"))
                .as_ref()
                .map(|s| s.as_str())
        );
    }

    #[test]
    fn test_array() {
        let keyed = array(imported("foo", "Debug"));

        let mut toks: Tokens<Go> = Tokens::new();
        toks.push(toks!(&keyed));

        assert_eq!(
            Ok("package foo\n\nimport \"foo\"\n\n[]foo.Debug\n"),
            toks.to_file_with(Config::from_package("foo"))
                .as_ref()
                .map(|s| s.as_str())
        );
    }
}
