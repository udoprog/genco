//! Specialization for Go code generation.

use std::collections::BTreeSet;
use std::fmt::{self, Write};
use {Cons, Custom, Formatter, Quoted, Tokens};

const SEP: &str = ".";

/// Name of an imported type.
#[derive(Debug, Clone)]
pub struct Name<'el> {
    /// Module of the imported name.
    module: Option<Cons<'el>>,
    /// Name imported.
    name: Cons<'el>,
}

/// Go token specialization.
#[derive(Debug, Clone)]
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
    fn type_imports<'a, 'b: 'a>(go: &'b Go<'b>, modules: &'a mut BTreeSet<&'b str>) {
        use self::Go::*;

        match *go {
            Type { ref name, .. } => {
                if let Some(module) = name.module.as_ref() {
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

    fn imports<'a>(tokens: &'a Tokens<'a, Self>) -> Option<Tokens<'a, Self>> {
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

    fn format(&self, out: &mut Formatter, extra: &mut Self::Extra, level: usize) -> fmt::Result {
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
                key.format(out, extra, level + 1)?;
                out.write_str("]")?;
                value.format(out, extra, level + 1)?;
            }
            Array { ref inner, .. } => {
                out.write_str("[")?;
                out.write_str("]")?;
                inner.format(out, extra, level + 1)?;
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
    use super::{array, imported, interface, map, Extra, Go};
    use {Quoted, Tokens};

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

    #[test]
    fn test_map() {
        let keyed = map(imported("foo", "Debug"), interface());

        let mut toks: Tokens<Go> = Tokens::new();
        toks.push(toks!(&keyed));

        assert_eq!(
            Ok("package foo\n\nimport \"foo\"\n\nmap[foo.Debug]interface{}\n"),
            toks.to_file_with(Extra::from_package("foo"))
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
            toks.to_file_with(Extra::from_package("foo"))
                .as_ref()
                .map(|s| s.as_str())
        );
    }
}
