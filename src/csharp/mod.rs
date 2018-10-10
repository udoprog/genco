//! Specialization for Csharp code generation.

mod argument;
mod class;
mod constructor;
mod enum_;
mod field;
mod interface;
mod method;
mod modifier;
mod utils;

pub use self::argument::Argument;
pub use self::class::Class;
pub use self::constructor::Constructor;
pub use self::enum_::Enum;
pub use self::field::Field;
pub use self::interface::Interface;
pub use self::method::Method;
pub use self::modifier::Modifier;
pub use self::utils::BlockComment;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{self, Write};
use {Cons, Custom, Formatter, IntoTokens, Tokens};

static SYSTEM: &'static str = "System";
static SEP: &'static str = ".";

/// Boolean Type
pub const BOOLEAN: Csharp<'static> = Csharp::Simple {
    name: "bool",
    alias: "Boolean",
};

/// Byte Type.
pub const BYTE: Csharp<'static> = Csharp::Simple {
    name: "byte",
    alias: "Byte",
};

/// Signed Byte Type.
pub const SBYTE: Csharp<'static> = Csharp::Simple {
    name: "sbyte",
    alias: "SByte",
};

/// Decimal Type
pub const DECIMAL: Csharp<'static> = Csharp::Simple {
    name: "decimal",
    alias: "Decimal",
};

/// Float Type.
pub const SINGLE: Csharp<'static> = Csharp::Simple {
    name: "float",
    alias: "Single",
};

/// Double Type.
pub const DOUBLE: Csharp<'static> = Csharp::Simple {
    name: "double",
    alias: "Double",
};

/// Int16 Type.
pub const INT16: Csharp<'static> = Csharp::Simple {
    name: "short",
    alias: "Int16",
};

/// Uint16 Type.
pub const UINT16: Csharp<'static> = Csharp::Simple {
    name: "ushort",
    alias: "UInt16",
};

/// Int32 Type.
pub const INT32: Csharp<'static> = Csharp::Simple {
    name: "int",
    alias: "Int32",
};

/// Uint32 Type.
pub const UINT32: Csharp<'static> = Csharp::Simple {
    name: "uint",
    alias: "UInt32",
};

/// Int64 Type.
pub const INT64: Csharp<'static> = Csharp::Simple {
    name: "long",
    alias: "Int64",
};

/// UInt64 Type.
pub const UINT64: Csharp<'static> = Csharp::Simple {
    name: "ulong",
    alias: "UInt64",
};

/// A class.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type<'el> {
    /// namespace of the class.
    namespace: Cons<'el>,
    /// Name  of class.
    name: Cons<'el>,
    /// Path of class when nested.
    path: Vec<Cons<'el>>,
    /// Arguments of the class.
    arguments: Vec<Csharp<'el>>,
    /// Use as qualified type.
    qualified: bool,
}

/// Csharp token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Csharp<'el> {
    /// Simple type.
    Simple {
        /// The name of the simple type.
        name: &'static str,
        /// Their .NET aliases.
        alias: &'static str,
    },
    /// An array of some type.
    Array(Box<Csharp<'el>>),
    /// A struct of some type.
    Struct(Type<'el>),
    /// The special `void` type.
    Void,
    /// A class, with or without arguments, using from somewhere.
    Class(Type<'el>),
    /// An enum of some type.
    Enum(Type<'el>),
    /// A local name with no specific qualification.
    Local {
        /// Name of class.
        name: Cons<'el>,
    },
    /// Optional type.
    Optional(Box<Csharp<'el>>),
}

into_tokens_impl_from!(Csharp<'el>, Csharp<'el>);
into_tokens_impl_from!(&'el Csharp<'el>, Csharp<'el>);

/// Extra data for Csharp formatting.
#[derive(Debug, Default)]
pub struct Extra<'el> {
    /// namespace to use.
    pub namespace: Option<Cons<'el>>,

    /// Names which have been imported (namespace + name).
    imported_names: HashMap<String, String>,
}

impl<'el> Extra<'el> {
    /// Set the namespace name to build.
    pub fn namespace<P>(&mut self, namespace: P)
    where
        P: Into<Cons<'el>>,
    {
        self.namespace = Some(namespace.into())
    }
}

impl<'el> Csharp<'el> {
    /// Extend the type with a nested path.
    ///
    /// This discards any arguments associated with it.
    pub fn path<P: Into<Cons<'el>>>(&self, part: P) -> Csharp<'el> {
        use self::Csharp::*;

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

    fn inner_imports<'a>(ty: &'a Type<'a>, modules: &mut BTreeSet<(&'a str, &'a str)>) {
        for argument in &ty.arguments {
            Self::type_imports(argument, modules);
        }

        modules.insert((ty.namespace.as_ref(), ty.name.as_ref()));
    }

    fn type_imports<'a>(csharp: &'a Csharp<'a>, modules: &mut BTreeSet<(&'a str, &'a str)>) {
        use self::Csharp::*;

        match *csharp {
            Simple { alias, .. } => {
                modules.insert((SYSTEM, alias));
            }
            Class(ref inner) => {
                Self::inner_imports(inner, modules);
            }
            Array(ref inner) => {
                Self::type_imports(inner, modules);
            }
            Struct(ref inner) => {
                Self::inner_imports(inner, modules);
            }
            Enum(ref inner) => {
                Self::inner_imports(inner, modules);
            }
            Optional(ref value) => {
                Self::type_imports(value, modules);
            }
            _ => {}
        };
    }

    fn imports<'a>(tokens: &'a Tokens<'a, Self>, extra: &mut Extra) -> Option<Tokens<'a, Self>> {
        let mut modules = BTreeSet::new();

        let file_namespace = extra.namespace.as_ref().map(|p| p.as_ref());

        for custom in tokens.walk_custom() {
            Self::type_imports(custom, &mut modules);
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();
        let mut imported = HashSet::new();

        for (namespace, name) in modules {
            if Some(namespace) == file_namespace {
                continue;
            }

            match extra.imported_names.get(name) {
                // already imported...
                Some(existing) if existing == namespace => continue,
                // already imported, as something else...
                Some(_) => continue,
                _ => {}
            }

            if !imported.contains(namespace) {
                out.push(toks!("using ", namespace, ";"));
                imported.insert(namespace.to_string());
            }

            extra
                .imported_names
                .insert(name.to_string(), namespace.to_string());
        }

        Some(out)
    }

    /// Add arguments to the given variable.
    ///
    /// Only applies to classes, any other will return the same value.
    pub fn with_arguments(&self, arguments: Vec<Csharp<'el>>) -> Csharp<'el> {
        use self::Csharp::*;

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
    pub fn into_struct(self) -> Csharp<'el> {
        use self::Csharp::*;

        match self {
            Class(inner) => Struct(inner),
            csharp => csharp,
        }
    }

    /// Convert this type into an enum.
    pub fn into_enum(self) -> Csharp<'el> {
        use self::Csharp::*;

        match self {
            Class(inner) => Enum(inner),
            csharp => csharp,
        }
    }

    /// Make this type into a qualified type that is always used with a namespace.
    pub fn qualified(self) -> Csharp<'el> {
        use self::Csharp::*;

        match self {
            Class(cls) => Class(Type {
                qualified: true,
                ..cls
            }),
            csharp => csharp,
        }
    }

    /// Compare if two types are equal.
    pub fn equals(&self, other: &Csharp<'el>) -> bool {
        use self::Csharp::*;

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
    pub fn name(&self) -> Cons<'el> {
        use self::Csharp::*;

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
    pub fn namespace(&self) -> Option<Cons<'el>> {
        use self::Csharp::*;

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
    pub fn arguments(&self) -> Option<&[Csharp<'el>]> {
        use self::Csharp::*;

        match *self {
            Class(ref inner) => Some(&inner.arguments),
            Optional(ref value) => value.arguments(),
            _ => None,
        }
    }

    /// Get the value type (strips optionality).
    pub fn as_value(&self) -> Csharp<'el> {
        self.as_optional()
            .map(|opt| opt.clone())
            .unwrap_or_else(|| self.clone())
    }

    /// Check if type is optional.
    pub fn is_optional(&self) -> bool {
        use self::Csharp::*;

        match *self {
            Optional(_) => true,
            _ => false,
        }
    }

    /// Check if type is nullable.
    pub fn is_nullable(&self) -> bool {
        use self::Csharp::*;

        match *self {
            Enum(_) | Struct(_) | Simple { .. } => false,
            _ => true,
        }
    }

    /// Check if variable is simple.
    pub fn is_simple(&self) -> bool {
        use self::Csharp::*;

        match *self {
            Simple { .. } => true,
            _ => false,
        }
    }

    /// Check if type is array.
    pub fn is_array(&self) -> bool {
        use self::Csharp::*;

        match *self {
            Array(_) => true,
            _ => false,
        }
    }

    /// Check if type is struct.
    pub fn is_struct(&self) -> bool {
        use self::Csharp::*;

        match *self {
            Struct(_) => true,
            _ => false,
        }
    }

    /// Check if type is an enum.
    pub fn is_enum(&self) -> bool {
        use self::Csharp::*;

        match *self {
            Enum(_) => true,
            _ => false,
        }
    }

    /// Get type as optional.
    pub fn as_optional(&self) -> Option<&Csharp<'el>> {
        use self::Csharp::*;

        match *self {
            Optional(ref optional) => Some(optional),
            _ => None,
        }
    }

    fn inner_format(
        &self,
        inner: &Type<'el>,
        out: &mut Formatter,
        extra: &mut <Self as Custom>::Extra,
        level: usize,
    ) -> fmt::Result {
        {
            let qualified = match inner.qualified {
                true => true,
                false => {
                    let file_namespace = extra.namespace.as_ref().map(|p| p.as_ref());
                    let imported = extra
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
                argument.format(out, extra, level + 1usize)?;

                if it.peek().is_some() {
                    out.write_str(", ")?;
                }
            }

            out.write_str(">")?;
        }

        Ok(())
    }
}

impl<'el> Custom for Csharp<'el> {
    type Extra = Extra<'el>;

    fn format(&self, out: &mut Formatter, extra: &mut Self::Extra, level: usize) -> fmt::Result {
        use self::Csharp::*;

        match *self {
            Simple { ref alias, .. } => {
                out.write_str(alias.as_ref())?;
            }
            Array(ref inner) => {
                inner.format(out, extra, level)?;
                out.write_str("[]")?;
            }
            Void => {
                out.write_str("void")?;
            }
            Enum(ref inner) | Struct(ref inner) | Class(ref inner) => {
                self.inner_format(inner, out, extra, level)?;
            }
            Local { ref name } => {
                out.write_str(name.as_ref())?;
            }
            Optional(ref value) => {
                value.format(out, extra, level)?;

                if !value.is_nullable() {
                    out.write_str("?")?;
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

        if let Some(imports) = Self::imports(&tokens, extra) {
            toks.push(imports);
        }

        if let Some(ref namespace) = extra.namespace {
            toks.push({
                let mut t = Tokens::new();

                t.push(toks!["namespace ", namespace.clone(), " {"]);
                t.nested_ref(&tokens);
                t.push("}");

                t
            });
        } else {
            toks.push_ref(&tokens);
        }

        toks.join_line_spacing().format(out, extra, level)
    }
}

/// Setup an imported element.
pub fn using<'a, P: Into<Cons<'a>>, N: Into<Cons<'a>>>(namespace: P, name: N) -> Csharp<'a> {
    Csharp::Class(Type {
        namespace: namespace.into(),
        name: name.into(),
        path: vec![],
        arguments: vec![],
        qualified: false,
    })
}

/// Setup a struct type.
pub fn struct_<'el, I: Into<Csharp<'el>>>(value: I) -> Csharp<'el> {
    value.into().into_struct()
}

/// Setup a local element from borrowed components.
pub fn local<'el, N: Into<Cons<'el>>>(name: N) -> Csharp<'el> {
    Csharp::Local { name: name.into() }
}

/// Setup an array type.
pub fn array<'el, I: Into<Csharp<'el>>>(value: I) -> Csharp<'el> {
    Csharp::Array(Box::new(value.into()))
}

/// Setup an optional type.
pub fn optional<'el, I: Into<Csharp<'el>>>(value: I) -> Csharp<'el> {
    Csharp::Optional(Box::new(value.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use csharp::Csharp;
    use quoted::Quoted;
    use tokens::Tokens;

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

        let toks = toks![a, b, ob, ob_a].join_spacing();

        assert_eq!(
            Ok("using Foo.Bar;\n\nA B Foo.Baz.B Foo.Baz.B<A>\n"),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
