//! Specialization for Dart code generation.

mod modifier;
mod utils;

pub use self::modifier::Modifier;
pub use self::utils::DocComment;

use super::cons::Cons;
use super::custom::Custom;
use super::formatter::Formatter;
use super::into_tokens::IntoTokens;
use super::tokens::Tokens;
use std::fmt::{self, Write};

static SEP: &'static str = ".";
pub static DART_CORE: &'static str = "dart:core";

/// Integer built-in type.
pub const INT: Dart<'static> = Dart::BuiltIn { name: "int" };

/// Double built-in type.
pub const DOUBLE: Dart<'static> = Dart::BuiltIn { name: "double" };

/// Boolean built-in type.
pub const BOOL: Dart<'static> = Dart::BuiltIn { name: "bool" };

/// All information about a single type.
#[derive(Default, Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type<'el> {
    /// Path to import.
    path: Option<Cons<'el>>,
    /// Alias of module.
    alias: Option<Cons<'el>>,
    /// Name imported.
    name: Option<Cons<'el>>,
    /// Generic arguments.
    arguments: Vec<Dart<'el>>,
}

/// Dart token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Dart<'el> {
    /// built-in type.
    BuiltIn {
        /// The built-in type.
        name: &'static str,
    },
    /// the void type.
    Void,
    /// the dynamic type.
    Dynamic,
    /// referenced types.
    Type(Type<'el>),
}

into_tokens_impl_from!(Dart<'el>, Dart<'el>);
into_tokens_impl_from!(&'el Dart<'el>, Dart<'el>);

/// Extra data for Dart formatting.
#[derive(Debug, Default)]
pub struct Extra {}

impl Extra {}

impl<'el> Dart<'el> {
    /// Resolve all imports.
    fn imports<'a, 'b: 'a>(
        input: &'b Tokens<'a, Dart<'el>>,
        _: &mut Extra,
    ) -> Tokens<'a, Dart<'el>> {
        use quoted::Quoted;
        use std::collections::BTreeSet;

        let mut modules = BTreeSet::new();

        for custom in input.walk_custom() {
            if let Dart::Type(ref ty) = *custom {
                if let Some(path) = ty.path.as_ref() {
                    if path.as_ref() == DART_CORE {
                        continue;
                    }

                    modules.insert((path.as_ref(), ty.alias.as_ref().map(AsRef::as_ref)));
                }
            }
        }

        if modules.is_empty() {
            return toks!();
        }

        let mut o = toks!();

        for (name, alias) in modules {
            if let Some(alias) = alias {
                o.push(toks!("import ", name.quoted(), " as ", alias, ";"));
            } else {
                o.push(toks!("import ", name.quoted(), ";"));
            }
        }

        return o;
    }

    /// Change the imported alias for this type.
    pub fn alias(&self, alias: impl Into<Cons<'el>>) -> Dart<'el> {
        match *self {
            Dart::Type(ref ty) => Dart::Type(Type {
                alias: Some(alias.into()),
                ..ty.clone()
            }),
            ref dart => dart.clone(),
        }
    }

    /// Change the imported name for this type.
    pub fn name(&self, name: impl Into<Cons<'el>>) -> Dart<'el> {
        match *self {
            Dart::Type(ref ty) => Dart::Type(Type {
                name: Some(name.into()),
                ..ty.clone()
            }),
            ref dart => dart.clone(),
        }
    }

    /// Add arguments to the given variable.
    ///
    /// Only applies to classes, any other will return the same value.
    pub fn with_arguments(&self, arguments: Vec<Dart<'el>>) -> Dart<'el> {
        match *self {
            Dart::Type(ref ty) => Dart::Type(Type {
                arguments: arguments,
                ..ty.clone()
            }),
            ref dart => dart.clone(),
        }
    }

    /// Get the arguments.
    pub fn arguments(&self) -> Option<&[Dart<'el>]> {
        use self::Dart::*;

        match *self {
            Type(ref ty) => Some(&ty.arguments),
            _ => None,
        }
    }

    /// Check if variable is built-in.
    pub fn is_built_in(&self) -> bool {
        use self::Dart::*;

        match *self {
            BuiltIn { .. } => true,
            _ => false,
        }
    }

    /// Check if this type belongs to a core package.
    pub fn is_core(&self) -> bool {
        use self::Dart::*;

        let ty = match *self {
            Type(ref ty) => ty,
            BuiltIn { .. } => return true,
            Void => return true,
            Dynamic => return true,
        };

        match ty.path.as_ref() {
            Some(path) => path.as_ref() == DART_CORE,
            None => false,
        }
    }

    /// Check if type is generic.
    pub fn is_generic(&self) -> bool {
        self.arguments().map(|a| !a.is_empty()).unwrap_or(false)
    }
}

impl<'el> Custom for Dart<'el> {
    type Extra = Extra;

    fn format(&self, out: &mut Formatter, extra: &mut Self::Extra, level: usize) -> fmt::Result {
        use self::Dart::*;

        match *self {
            BuiltIn { ref name, .. } => {
                out.write_str(name.as_ref())?;
            }
            Void => out.write_str("void")?,
            Dynamic => out.write_str("dynamic")?,
            Type(ref ty) => {
                if let Some(ref name) = ty.name {
                    if let Some(ref alias) = ty.alias {
                        out.write_str(alias.as_ref())?;
                        out.write_str(SEP)?;
                    }

                    out.write_str(name.as_ref())?;

                    if !ty.arguments.is_empty() {
                        out.write_str("<")?;

                        let mut it = ty.arguments.iter().peekable();

                        while let Some(argument) = it.next() {
                            argument.format(out, extra, level + 1)?;

                            if it.peek().is_some() {
                                out.write_str(", ")?;
                            }
                        }

                        out.write_str(">")?;
                    }
                }
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

        toks.push_unless_empty(Self::imports(&tokens, extra));
        toks.push_ref(&tokens);
        toks.join_line_spacing().format(out, extra, level)
    }
}

/// Setup an imported element.
pub fn imported<'a, P: Into<Cons<'a>>>(path: P) -> Dart<'a> {
    Dart::Type(Type {
        path: Some(path.into()),
        ..Type::default()
    })
}

/// Setup a local element.
pub fn local<'el, N: Into<Cons<'el>>>(name: N) -> Dart<'el> {
    Dart::Type(Type {
        name: Some(name.into()),
        ..Type::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dart::Dart;
    use quoted::Quoted;
    use tokens::Tokens;

    #[test]
    fn test_builtin() {
        assert!(INTEGER.is_built_in());
        assert!(DOUBLE.is_built_in());
        assert!(BOOLEAN.is_built_in());
        assert!(!Dart::Void.is_built_in());
    }

    #[test]
    fn test_string() {
        let mut toks: Tokens<Dart> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }

    #[test]
    fn test_imported() {
        let import = imported("package:http/http.dart");
        let import2 = imported("package:http/http.dart");
        let import_alias = imported("package:http/http.dart").alias("h2");
        let import_relative = imported("../http.dart");

        let toks = toks![
            import.name("a"),
            import2.name("b"),
            import_alias.name("c"),
            import_relative.name("d"),
        ].join_spacing();

        let expected = vec![
            "import \"../http.dart\";",
            "import \"package:http/http.dart\";",
            "import \"package:http/http.dart\" as h2;",
            "",
            "a b h2.c d",
            "",
        ];

        assert_eq!(
            Ok(expected.join("\n").as_str()),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
