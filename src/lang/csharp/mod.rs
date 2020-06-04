//! Specialization for Csharp code generation.

mod modifier;
mod utils;

pub use self::modifier::Modifier;
pub use self::utils::BlockComment;
use crate::{Formatter, ItemStr, Lang, LangItem};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;

/// Tokens container specialization for C#.
pub type Tokens = crate::Tokens<Csharp>;

impl_type_basics!(Csharp, TypeEnum<'a>, TypeTrait, TypeBox, TypeArgs, {Simple, Optional, Type, Array, Void});

/// Trait implemented by all types
pub trait TypeTrait: 'static + fmt::Debug + LangItem<Csharp> {
    /// Coerce trait into an enum that can be used for type-specific operations
    fn as_enum(&self) -> TypeEnum<'_>;

    /// Get the name of the type.
    fn name(&self) -> &str;

    /// Get the namespace of the type, if available.
    fn namespace(&self) -> Option<&str> {
        None
    }

    /// Check if type is nullable.
    fn is_nullable(&self) -> bool;

    /// Handle imports recursively.
    fn type_imports(&self, _: &mut BTreeSet<(ItemStr, ItemStr)>) {}
}

static SYSTEM: &'static str = "System";
static SEP: &'static str = ".";

/// Boolean type
pub const BOOLEAN: Simple = Simple {
    name: "bool",
    alias: "Boolean",
};

/// Byte type.
pub const BYTE: Simple = Simple {
    name: "byte",
    alias: "Byte",
};

/// Signed Byte type.
pub const SBYTE: Simple = Simple {
    name: "sbyte",
    alias: "SByte",
};

/// Decimal type
pub const DECIMAL: Simple = Simple {
    name: "decimal",
    alias: "Decimal",
};

/// Float type.
pub const SINGLE: Simple = Simple {
    name: "float",
    alias: "Single",
};

/// Double type.
pub const DOUBLE: Simple = Simple {
    name: "double",
    alias: "Double",
};

/// Int16 type.
pub const INT16: Simple = Simple {
    name: "short",
    alias: "Int16",
};

/// Uint16 type.
pub const UINT16: Simple = Simple {
    name: "ushort",
    alias: "UInt16",
};

/// Int32 type.
pub const INT32: Simple = Simple {
    name: "int",
    alias: "Int32",
};

/// Uint32 type.
pub const UINT32: Simple = Simple {
    name: "uint",
    alias: "UInt32",
};

/// Int64 type.
pub const INT64: Simple = Simple {
    name: "long",
    alias: "Int64",
};

/// UInt64 type.
pub const UINT64: Simple = Simple {
    name: "ulong",
    alias: "UInt64",
};

/// The `void` type.
pub const VOID: Void = Void(());

/// Config data for Csharp formatting.
#[derive(Debug, Default)]
pub struct Config {
    /// namespace to use.
    namespace: Option<ItemStr>,

    /// Names which have been imported (namespace + name).
    imported_names: HashMap<String, String>,
}

impl Config {
    /// Set the namespace name to build.
    pub fn with_namespace<N>(self, namespace: N) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            namespace: Some(namespace.into()),
            ..self
        }
    }
}

/// An optional type.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Optional {
    inner: TypeBox,
}

impl TypeTrait for Optional {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Optional(self)
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn namespace(&self) -> Option<&str> {
        self.inner.namespace()
    }

    fn is_nullable(&self) -> bool {
        false
    }

    fn type_imports(&self, modules: &mut BTreeSet<(ItemStr, ItemStr)>) {
        self.inner.type_imports(modules)
    }
}

impl LangItem<Csharp> for Optional {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        self.inner.format(out, config, level)?;

        if !self.inner.is_nullable() {
            out.write_str("?")?;
        }

        Ok(())
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// The kind of the pointed to type.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Kind {
    /// The pointed to type is an enum.
    Enum,
    /// The pointed to type is a class.
    Class,
    /// The pointed to type is a struct.
    Struct,
}

/// A class.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type {
    /// namespace of the class.
    namespace: Option<ItemStr>,
    /// Name  of class.
    name: ItemStr,
    /// Path of class when nested.
    path: Vec<ItemStr>,
    /// Arguments of the class.
    arguments: Vec<TypeBox>,
    /// Use as qualified type.
    qualified: bool,
    /// The kind of the type.
    kind: Kind,
}

impl Type {
    /// Extend the type with a nested path.
    ///
    /// This discards any arguments associated with it.
    pub fn path<P: Into<ItemStr>>(self, part: P) -> Self {
        let mut path = self.path;
        path.push(part.into());

        Self {
            path: path,
            arguments: vec![],
            ..self
        }
    }

    /// Add arguments to the given type.
    ///
    /// Only applies to classes, any other will return the same value.
    pub fn with_arguments(self, args: impl TypeArgs) -> Self {
        Self {
            arguments: args.into_args(),
            ..self
        }
    }

    /// Make this type into a qualified type that is always used with a namespace.
    pub fn qualified(self) -> Self {
        Self {
            qualified: true,
            ..self
        }
    }

    /// Convert this type into a struct.
    pub fn into_struct(self) -> Self {
        Self {
            kind: Kind::Struct,
            arguments: vec![],
            ..self
        }
    }

    /// Convert this type into an enum.
    pub fn into_enum(self) -> Self {
        Self {
            kind: Kind::Enum,
            arguments: vec![],
            ..self
        }
    }
}

impl TypeTrait for Type {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Type(self)
    }

    fn name(&self) -> &str {
        &*self.name
    }

    fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    fn is_nullable(&self) -> bool {
        match self.kind {
            Kind::Enum | Kind::Struct => false,
            _ => true,
        }
    }

    fn type_imports(&self, modules: &mut BTreeSet<(ItemStr, ItemStr)>) {
        for argument in &self.arguments {
            argument.type_imports(modules);
        }

        if let Some(namespace) = &self.namespace {
            modules.insert((namespace.clone(), self.name.clone()));
        }
    }
}

impl LangItem<Csharp> for Type {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        {
            let qualified = match self.qualified {
                true => true,
                false => {
                    let file_namespace = config.namespace.as_ref().map(|p| p.as_ref());
                    let imported = config
                        .imported_names
                        .get(self.name.as_ref())
                        .map(String::as_str);
                    let pkg = self.namespace.as_deref();
                    imported != pkg && file_namespace != pkg
                }
            };

            if let Some(namespace) = &self.namespace {
                if qualified {
                    out.write_str(namespace)?;
                    out.write_str(SEP)?;
                }
            }
        }

        {
            out.write_str(self.name.as_ref())?;

            let mut it = self.path.iter();

            while let Some(n) = it.next() {
                out.write_str(".")?;
                out.write_str(n.as_ref())?;
            }
        }

        if !self.arguments.is_empty() {
            out.write_str("<")?;

            let mut it = self.arguments.iter().peekable();

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

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// Simple type.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Simple {
    /// The name of the simple type.
    name: &'static str,
    /// Their .NET aliases.
    alias: &'static str,
}

impl TypeTrait for Simple {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Simple(self)
    }

    fn name(&self) -> &str {
        self.name
    }

    fn namespace(&self) -> Option<&str> {
        Some(SYSTEM)
    }

    fn is_nullable(&self) -> bool {
        false
    }

    fn type_imports(&self, modules: &mut BTreeSet<(ItemStr, ItemStr)>) {
        modules.insert((SYSTEM.into(), self.alias.into()));
    }
}

impl LangItem<Csharp> for Simple {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str(self.alias)?;
        Ok(())
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// An array type in C#.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Array {
    inner: TypeBox,
}

impl TypeTrait for Array {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Array(self)
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn namespace(&self) -> Option<&str> {
        self.inner.namespace()
    }

    fn is_nullable(&self) -> bool {
        true
    }

    fn type_imports(&self, modules: &mut BTreeSet<(ItemStr, ItemStr)>) {
        self.inner.type_imports(modules);
    }
}

impl LangItem<Csharp> for Array {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        self.inner.format(out, config, level)?;
        out.write_str("[]")?;
        Ok(())
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// The special `void` type.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Void(());

impl TypeTrait for Void {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Void(self)
    }

    fn name(&self) -> &str {
        "void"
    }

    fn is_nullable(&self) -> bool {
        false
    }
}

impl LangItem<Csharp> for Void {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str("void")?;
        Ok(())
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// Language specialization for C#.
pub struct Csharp(());

impl Csharp {
    fn imports(tokens: &Tokens, config: &mut Config) -> Option<Tokens> {
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
    type Import = dyn TypeTrait;

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
        tokens: Tokens,
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
pub fn using<P: Into<ItemStr>, N: Into<ItemStr>>(namespace: P, name: N) -> Type {
    Type {
        namespace: Some(namespace.into()),
        name: name.into(),
        path: vec![],
        arguments: vec![],
        qualified: false,
        kind: Kind::Class,
    }
}

/// Setup a local element from borrowed components.
pub fn local<N: Into<ItemStr>>(name: N) -> Type {
    Type {
        namespace: None,
        name: name.into(),
        path: vec![],
        arguments: vec![],
        qualified: false,
        kind: Kind::Class,
    }
}

/// Setup an array type.
pub fn array<I: Into<TypeBox>>(value: I) -> Array {
    Array {
        inner: value.into(),
    }
}

/// Setup an optional type.
pub fn optional<I: Into<TypeBox>>(value: I) -> Optional {
    Optional {
        inner: value.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as genco;
    use crate::{quote, Csharp, Ext as _, Tokens};

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
        let ob_a = ob.clone().with_arguments(a.clone());

        let toks: Tokens<Csharp> = quote!(#a #b #ob #ob_a);

        assert_eq!(
            vec!["using Foo.Bar;", "", "A B Foo.Baz.B Foo.Baz.B<A>", ""],
            toks.to_file_vec().unwrap()
        );
    }
}
