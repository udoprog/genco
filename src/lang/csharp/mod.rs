//! Specialization for Csharp code generation.

mod modifier;
mod utils;

pub use self::modifier::Modifier;
pub use self::utils::BlockComment;
use crate::{Cons, Formatter, Lang, LangItem};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;

/// Tokens container specialization for C#.
pub type Tokens<'el> = crate::Tokens<'el, Csharp>;

static SYSTEM: &'static str = "System";
static SEP: &'static str = ".";

/// Boolean Type
pub const BOOLEAN: Imported = Imported::Simple {
    name: "bool",
    alias: "Boolean",
};

/// Byte Type.
pub const BYTE: Imported = Imported::Simple {
    name: "byte",
    alias: "Byte",
};

/// Signed Byte Type.
pub const SBYTE: Imported = Imported::Simple {
    name: "sbyte",
    alias: "SByte",
};

/// Decimal Type
pub const DECIMAL: Imported = Imported::Simple {
    name: "decimal",
    alias: "Decimal",
};

/// Float Type.
pub const SINGLE: Imported = Imported::Simple {
    name: "float",
    alias: "Single",
};

/// Double Type.
pub const DOUBLE: Imported = Imported::Simple {
    name: "double",
    alias: "Double",
};

/// Int16 Type.
pub const INT16: Imported = Imported::Simple {
    name: "short",
    alias: "Int16",
};

/// Uint16 Type.
pub const UINT16: Imported = Imported::Simple {
    name: "ushort",
    alias: "UInt16",
};

/// Int32 Type.
pub const INT32: Imported = Imported::Simple {
    name: "int",
    alias: "Int32",
};

/// Uint32 Type.
pub const UINT32: Imported = Imported::Simple {
    name: "uint",
    alias: "UInt32",
};

/// Int64 Type.
pub const INT64: Imported = Imported::Simple {
    name: "long",
    alias: "Int64",
};

/// UInt64 Type.
pub const UINT64: Imported = Imported::Simple {
    name: "ulong",
    alias: "UInt64",
};

/// A class.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type {
    /// namespace of the class.
    namespace: Cons<'static>,
    /// Name  of class.
    name: Cons<'static>,
    /// Path of class when nested.
    path: Vec<Cons<'static>>,
    /// Arguments of the class.
    arguments: Vec<Imported>,
    /// Use as qualified type.
    qualified: bool,
}

impl Type {
    /// Handle type imports.
    fn type_import(&self, modules: &mut BTreeSet<(Cons<'static>, Cons<'static>)>) {
        for argument in &self.arguments {
            argument.type_imports(modules);
        }

        modules.insert((self.namespace.clone(), self.name.clone()));
    }
}

/// Csharp token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Imported {
    /// Simple type.
    Simple {
        /// The name of the simple type.
        name: &'static str,
        /// Their .NET aliases.
        alias: &'static str,
    },
    /// An array of some type.
    Array(Box<Imported>),
    /// A struct of some type.
    Struct(Type),
    /// The special `void` type.
    Void,
    /// A class, with or without arguments, using from somewhere.
    Class(Type),
    /// An enum of some type.
    Enum(Type),
    /// A local name with no specific qualification.
    Local {
        /// Name of class.
        name: Cons<'static>,
    },
    /// Optional type.
    Optional(Box<Imported>),
}

/// Config data for Csharp formatting.
#[derive(Debug, Default)]
pub struct Config {
    /// namespace to use.
    pub namespace: Option<Cons<'static>>,

    /// Names which have been imported (namespace + name).
    imported_names: HashMap<String, String>,
}

impl Config {
    /// Set the namespace name to build.
    pub fn namespace<P>(&mut self, namespace: P)
    where
        P: Into<Cons<'static>>,
    {
        self.namespace = Some(namespace.into())
    }
}

impl Imported {
    /// Extend the type with a nested path.
    ///
    /// This discards any arguments associated with it.
    pub fn path<P: Into<Cons<'static>>>(&self, part: P) -> Imported {
        use self::Imported::*;

        match *self {
            Class(ref class) => {
                let mut path = class.path.clone();
                path.push(part.into());

                Class(Type {
                    namespace: class.namespace.clone(),
                    name: class.name.clone(),
                    path: path,
                    arguments: vec![],
                    qualified: class.qualified,
                })
            }
            ref csharp => csharp.clone(),
        }
    }

    fn type_imports(&self, modules: &mut BTreeSet<(Cons<'static>, Cons<'static>)>) {
        match self {
            Self::Simple { alias, .. } => {
                modules.insert((SYSTEM.into(), (*alias).into()));
            }
            Self::Class(inner) | Self::Struct(inner) | Self::Enum(inner) => {
                inner.type_import(modules);
            }
            Self::Array(ty) | Self::Optional(ty) => {
                ty.type_imports(modules);
            }
            _ => {}
        };
    }

    /// Add arguments to the given variable.
    ///
    /// Only applies to classes, any other will return the same value.
    pub fn with_arguments(&self, arguments: Vec<Imported>) -> Imported {
        use self::Imported::*;

        match *self {
            Class(ref cls) => Class(Type {
                namespace: cls.namespace.clone(),
                name: cls.name.clone(),
                path: cls.path.clone(),
                arguments: arguments,
                qualified: cls.qualified,
            }),
            ref csharp => csharp.clone(),
        }
    }

    /// Convert this type into a struct.
    pub fn into_struct(self) -> Imported {
        use self::Imported::*;

        match self {
            Class(inner) => Struct(inner),
            csharp => csharp,
        }
    }

    /// Convert this type into an enum.
    pub fn into_enum(self) -> Imported {
        use self::Imported::*;

        match self {
            Class(inner) => Enum(inner),
            csharp => csharp,
        }
    }

    /// Make this type into a qualified type that is always used with a namespace.
    pub fn qualified(self) -> Imported {
        use self::Imported::*;

        match self {
            Class(cls) => Class(Type {
                qualified: true,
                ..cls
            }),
            csharp => csharp,
        }
    }

    /// Compare if two types are equal.
    pub fn equals(&self, other: &Imported) -> bool {
        use self::Imported::*;

        match (self, other) {
            (
                &Simple {
                    name: ref l_name, ..
                },
                &Simple {
                    name: ref r_name, ..
                },
            ) => l_name == r_name,
            (&Class(ref l), &Class(ref r)) => {
                l.namespace == r.namespace
                    && l.name == r.name
                    && l.arguments.len() == r.arguments.len()
                    && l.arguments
                        .iter()
                        .zip(r.arguments.iter())
                        .all(|(l, r)| l.equals(r))
            }
            _ => false,
        }
    }

    /// Get the name of the type.
    pub fn name(&self) -> Cons<'static> {
        use self::Imported::*;

        match *self {
            Simple { ref name, .. } => Cons::Borrowed(name),
            Enum(ref inner) | Struct(ref inner) | Class(ref inner) => inner.name.clone(),
            Local { ref name, .. } => name.clone(),
            Optional(ref value) => value.name(),
            Array(ref inner) => inner.name(),
            Void => Cons::Borrowed("void"),
        }
    }

    /// Get the name of the type.
    pub fn namespace(&self) -> Option<Cons<'static>> {
        use self::Imported::*;

        match *self {
            Simple { .. } => Some(Cons::Borrowed(SYSTEM)),
            Enum(ref inner) | Struct(ref inner) | Class(ref inner) => Some(inner.namespace.clone()),
            Local { .. } => None,
            Optional(ref value) => value.namespace(),
            Array(ref inner) => inner.namespace(),
            Void => None,
        }
    }

    /// Get the arguments
    pub fn arguments(&self) -> Option<&[Imported]> {
        use self::Imported::*;

        match *self {
            Class(ref inner) => Some(&inner.arguments),
            Optional(ref value) => value.arguments(),
            _ => None,
        }
    }

    /// Get the value type (strips optionality).
    pub fn as_value(&self) -> Imported {
        self.as_optional()
            .map(|opt| opt.clone())
            .unwrap_or_else(|| self.clone())
    }

    /// Check if type is optional.
    pub fn is_optional(&self) -> bool {
        use self::Imported::*;

        match *self {
            Optional(_) => true,
            _ => false,
        }
    }

    /// Check if type is nullable.
    pub fn is_nullable(&self) -> bool {
        use self::Imported::*;

        match *self {
            Enum(_) | Struct(_) | Simple { .. } => false,
            _ => true,
        }
    }

    /// Check if variable is simple.
    pub fn is_simple(&self) -> bool {
        use self::Imported::*;

        match *self {
            Simple { .. } => true,
            _ => false,
        }
    }

    /// Check if type is array.
    pub fn is_array(&self) -> bool {
        use self::Imported::*;

        match *self {
            Array(_) => true,
            _ => false,
        }
    }

    /// Check if type is struct.
    pub fn is_struct(&self) -> bool {
        use self::Imported::*;

        match *self {
            Struct(_) => true,
            _ => false,
        }
    }

    /// Check if type is an enum.
    pub fn is_enum(&self) -> bool {
        use self::Imported::*;

        match *self {
            Enum(_) => true,
            _ => false,
        }
    }

    /// Get type as optional.
    pub fn as_optional(&self) -> Option<&Imported> {
        use self::Imported::*;

        match *self {
            Optional(ref optional) => Some(optional),
            _ => None,
        }
    }

    fn inner_format(
        &self,
        inner: &Type,
        out: &mut Formatter,
        config: &mut Config,
        level: usize,
    ) -> fmt::Result {
        {
            let qualified = match inner.qualified {
                true => true,
                false => {
                    let file_namespace = config.namespace.as_ref().map(|p| p.as_ref());
                    let imported = config
                        .imported_names
                        .get(inner.name.as_ref())
                        .map(String::as_str);
                    let pkg = Some(inner.namespace.as_ref());
                    imported != pkg && file_namespace != pkg
                }
            };

            if qualified {
                out.write_str(inner.namespace.as_ref())?;
                out.write_str(SEP)?;
            }
        }

        {
            out.write_str(inner.name.as_ref())?;

            let mut it = inner.path.iter();

            while let Some(n) = it.next() {
                out.write_str(".")?;
                out.write_str(n.as_ref())?;
            }
        }

        if !inner.arguments.is_empty() {
            out.write_str("<")?;

            let mut it = inner.arguments.iter().peekable();

            while let Some(argument) = it.next() {
                argument.format(out, config, level + 1usize)?;

                if it.peek().is_some() {
                    out.write_str(", ")?;
                }
            }

            out.write_str(">")?;
        }

        Ok(())
    }
}

impl LangItem<Csharp> for Imported {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        use self::Imported::*;

        match *self {
            Simple { ref alias, .. } => {
                out.write_str(alias.as_ref())?;
            }
            Array(ref inner) => {
                inner.format(out, config, level)?;
                out.write_str("[]")?;
            }
            Void => {
                out.write_str("void")?;
            }
            Enum(ref inner) | Struct(ref inner) | Class(ref inner) => {
                self.inner_format(inner, out, config, level)?;
            }
            Local { ref name } => {
                out.write_str(name.as_ref())?;
            }
            Optional(ref value) => {
                value.format(out, config, level)?;

                if !value.is_nullable() {
                    out.write_str("?")?;
                }
            }
        }

        Ok(())
    }

    fn as_import(&self) -> Option<&Self> {
        Some(self)
    }
}

impl_lang_item!(Imported, Csharp);

/// Language specialization for C#.
pub struct Csharp(());

impl Csharp {
    fn imports<'el>(tokens: &Tokens<'el>, config: &mut Config) -> Option<Tokens<'el>> {
        let mut modules = BTreeSet::new();

        let file_namespace = config.namespace.as_ref().map(|p| p.as_ref());

        for custom in tokens.walk_custom() {
            if let Some(import) = custom.as_import() {
                import.type_imports(&mut modules);
            }
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();
        let mut imported = HashSet::new();

        for (namespace, name) in modules {
            if Some(&*namespace) == file_namespace.as_deref() {
                continue;
            }

            match config.imported_names.get(&*name) {
                // already imported...
                Some(existing) if existing == &*namespace => continue,
                // already imported, as something else...
                Some(_) => continue,
                _ => {}
            }

            if !imported.contains(&*namespace) {
                out.push(toks!("using ", namespace.clone(), ";"));
                imported.insert(namespace.to_string());
            }

            config
                .imported_names
                .insert(name.to_string(), namespace.to_string());
        }

        Some(out)
    }
}

impl Lang for Csharp {
    type Config = Config;
    type Import = Imported;

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        use std::fmt::Write as _;

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

    fn write_file(
        tokens: Tokens<'_>,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks: Tokens = Tokens::new();

        if let Some(imports) = Self::imports(&tokens, config) {
            toks.push(imports);
            toks.line_spacing();
        }

        if let Some(ref namespace) = config.namespace {
            toks.push(toks!["namespace ", namespace.clone(), " {"]);
            toks.indent();
            toks.append(tokens);
            toks.unindent();
            toks.push("}");
        } else {
            toks.append(tokens);
        }

        toks.format(out, config, level)
    }
}

/// Setup an imported element.
pub fn using<P: Into<Cons<'static>>, N: Into<Cons<'static>>>(namespace: P, name: N) -> Imported {
    Imported::Class(Type {
        namespace: namespace.into(),
        name: name.into(),
        path: vec![],
        arguments: vec![],
        qualified: false,
    })
}

/// Setup a struct type.
pub fn struct_<I: Into<Imported>>(value: I) -> Imported {
    value.into().into_struct()
}

/// Setup a local element from borrowed components.
pub fn local<N: Into<Cons<'static>>>(name: N) -> Imported {
    Imported::Local { name: name.into() }
}

/// Setup an array type.
pub fn array<I: Into<Imported>>(value: I) -> Imported {
    Imported::Array(Box::new(value.into()))
}

/// Setup an optional type.
pub fn optional<I: Into<Imported>>(value: I) -> Imported {
    Imported::Optional(Box::new(value.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as genco;
    use crate::{quote, Csharp, Quoted, Tokens};

    #[test]
    fn test_simple() {
        assert!(BOOLEAN.is_simple());
        assert!(BYTE.is_simple());
        assert!(SBYTE.is_simple());
        assert!(DECIMAL.is_simple());
        assert!(SINGLE.is_simple());
        assert!(DOUBLE.is_simple());
        assert!(INT16.is_simple());
        assert!(UINT16.is_simple());
        assert!(INT32.is_simple());
        assert!(UINT32.is_simple());
        assert!(INT64.is_simple());
        assert!(UINT64.is_simple());
    }

    #[test]
    fn test_string() {
        let mut toks: Tokens<Csharp> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }

    #[ignore]
    #[test]
    fn test_using() {
        let a = using("Foo.Bar", "A");
        let b = using("Foo.Bar", "B");
        let ob = using("Foo.Baz", "B");
        let ob_a = ob.with_arguments(vec![a.clone()]);

        let toks: Tokens<Csharp> = quote!(#a #b #ob #ob_a);

        assert_eq!(
            vec!["using Foo.Bar;", "", "A B Foo.Baz.B Foo.Baz.B<A>", ""],
            toks.to_file_vec().unwrap()
        );
    }
}
