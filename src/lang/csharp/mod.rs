//! Specialization for Csharp code generation.
//!
//! # String Quoting in C#
//!
//! Since C# uses UTF-16 internally, but literal strings support C-style family
//! of escapes.
//!
//! See [c_family_escape][super::c_family_escape].
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: csharp::Tokens = quote!("start π 😊 \n \x7f end");
//! assert_eq!("\"start \\u03c0 \\U0001f60a \\n \\x7f end\"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```

mod block_comment;
mod comment;

use crate as genco;
use crate::fmt;
use crate::lang::{Lang, LangItem};
use crate::quote_in;
use crate::tokens::ItemStr;
use std::collections::{BTreeSet, HashMap, HashSet};

pub use self::block_comment::BlockComment;
pub use self::comment::Comment;

/// Tokens container specialization for C#.
pub type Tokens = crate::Tokens<Csharp>;

impl_dynamic_types! { Csharp =>
    pub trait TypeTrait {
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

    pub trait Args;
    pub struct Any;
    pub enum AnyRef;

    impl TypeTrait for Simple {
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

    impl TypeTrait for Optional {
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

    impl TypeTrait for Type {
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

    impl TypeTrait for Array {
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

    impl TypeTrait for Void {
        fn name(&self) -> &str {
            "void"
        }

        fn is_nullable(&self) -> bool {
            false
        }
    }
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

impl_modifier! {
    /// A Csharp modifier.
    ///
    /// A vector of modifiers have a custom implementation, allowing them to be
    /// formatted with a spacing between them in the language-recommended order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use csharp::Modifier::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let toks: csharp::Tokens = quote!(#(vec![Async, Static, Public]));
    ///
    /// assert_eq!("public async static", toks.to_string()?);
    /// # Ok(())
    /// # }
    /// ```
    pub enum Modifier<Csharp> {
        /// The `public` modifier.
        Public => "public",
        /// The `private` modifier.
        Private => "private",
        /// The `internal` modifier.
        Internal => "internal",
        /// The `protected` modifier.
        Protected => "protected",
        /// The `abstract` modifier.
        Abstract => "abstract",
        /// The `async` modifier.
        Async => "async",
        /// The `const` modifier.
        Const => "const",
        /// The `event` modifier.
        Event => "event",
        /// The `extern` modifier.
        Extern => "extern",
        /// The `new` modifier.
        New => "new",
        /// The `override` modifier.
        Override => "override",
        /// The `partial` modifier.
        Partial => "partial",
        /// The `readonly` modifier.
        Readonly => "readonly",
        /// The `sealed` modifier.
        Sealed => "sealed",
        /// The `static` modifier.
        Static => "static",
        /// The `unsafe` modifier.
        Unsafe => "unsafe",
        /// The `virtual` modifier.
        Virtual => "virtual",
        /// The `volatile` modifier.
        Volatile => "volatile",
    }
}

/// State using during formatting of C# language items.
#[derive(Debug, Default)]
pub struct Format {
    /// Keeping track of names which have been imported, do determine whether
    /// their use has to be qualified or not.
    ///
    /// A missing name means that it has to be used in a qualified manner.
    imported_names: HashMap<String, String>,
}

/// Config data for Csharp formatting.
#[derive(Debug, Default)]
pub struct Config {
    /// namespace to use.
    namespace: Option<ItemStr>,
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
    inner: Any,
}

impl_lang_item! {
    impl LangItem<Csharp> for Optional {
        fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
            self.inner.format(out, config, format)?;

            if !self.inner.is_nullable() {
                out.write_str("?")?;
            }

            Ok(())
        }

        fn as_import(&self) -> Option<&dyn TypeTrait> {
            Some(self)
        }
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
    arguments: Vec<Any>,
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
    pub fn with_arguments(self, args: impl Args) -> Self {
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

impl_lang_item! {
    impl LangItem<Csharp> for Type {
        fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
            {
                let qualified = match self.qualified {
                    true => true,
                    false => {
                        let file_namespace = config.namespace.as_ref().map(|p| p.as_ref());
                        let imported = format
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
                    argument.format(out, config, format)?;

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
}

/// Simple type.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Simple {
    /// The name of the simple type.
    name: &'static str,
    /// Their .NET aliases.
    alias: &'static str,
}

impl_lang_item! {
    impl LangItem<Csharp> for Simple {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str(self.alias)?;
            Ok(())
        }

        fn as_import(&self) -> Option<&dyn TypeTrait> {
            Some(self)
        }
    }
}

/// An array type in C#.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Array {
    inner: Any,
}

impl_lang_item! {
    impl LangItem<Csharp> for Array {
        fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
            self.inner.format(out, config, format)?;
            out.write_str("[]")?;
            Ok(())
        }

        fn as_import(&self) -> Option<&dyn TypeTrait> {
            Some(self)
        }
    }
}

/// The special `void` type.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Void(());

impl_lang_item! {
    impl LangItem<Csharp> for Void {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str("void")?;
            Ok(())
        }

        fn as_import(&self) -> Option<&dyn TypeTrait> {
            Some(self)
        }
    }
}

/// Language specialization for C#.
pub struct Csharp(());

impl Csharp {
    fn imports(
        out: &mut Tokens,
        tokens: &Tokens,
        config: &Config,
        imported_names: &mut HashMap<String, String>,
    ) {
        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            import.type_imports(&mut modules);
        }

        if modules.is_empty() {
            return;
        }

        let mut imported = HashSet::new();

        for (namespace, name) in modules {
            if Some(&*namespace) == config.namespace.as_deref() {
                continue;
            }

            match imported_names.get(&*name) {
                // already imported...
                Some(existing) if existing == &*namespace => continue,
                // already imported, as something else...
                Some(_) => continue,
                _ => {}
            }

            if !imported.contains(&*namespace) {
                quote_in!(*out => using #(&namespace););
                out.push();
                imported.insert(namespace.to_string());
            }

            imported_names.insert(name.to_string(), namespace.to_string());
        }

        out.line();
    }
}

impl Lang for Csharp {
    type Config = Config;
    type Format = Format;
    type Import = dyn TypeTrait;

    fn quote_string(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
        // From: https://csharpindepth.com/articles/Strings
        super::c_family_escape(out, input)
    }

    fn format_file(
        tokens: &Tokens,
        out: &mut fmt::Formatter<'_>,
        config: &Self::Config,
    ) -> fmt::Result {
        let mut file: Tokens = Tokens::new();

        let mut format = Format::default();

        Self::imports(&mut file, tokens, config, &mut format.imported_names);

        if let Some(namespace) = &config.namespace {
            quote_in! { file =>
                namespace #namespace {
                    #tokens
                }
            }

            file.format(out, config, &format)?;
        } else {
            file.format(out, config, &format)?;
            tokens.format(out, config, &format)?;
        }

        Ok(())
    }
}

/// Construct an imported type.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let a = csharp::using("Foo.Bar", "A");
/// let b = csharp::using("Foo.Bar", "B");
/// let ob = csharp::using("Foo.Baz", "B");
/// let ob_a = ob.clone().with_arguments(a.clone());
///
/// let toks: Tokens<Csharp> = quote! {
///     #a
///     #b
///     #ob
///     #ob_a
/// };
///
/// assert_eq!(
///     vec![
///         "using Foo.Bar;",
///         "",
///         "A",
///         "B",
///         "Foo.Baz.B",
///         "Foo.Baz.B<A>",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
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
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let ty = csharp::array(csharp::using("Foo.Bar", "A"));
///
/// let toks: Tokens<Csharp> = quote! {
///     #ty
/// };
///
/// assert_eq!(
///     vec![
///         "using Foo.Bar;",
///         "",
///         "A[]",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
pub fn array<I: Into<Any>>(value: I) -> Array {
    Array {
        inner: value.into(),
    }
}

/// Setup an optional type.
pub fn optional<I: Into<Any>>(value: I) -> Optional {
    Optional {
        inner: value.into(),
    }
}

/// Format a doc comment where each line is preceeded by `///`.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
/// use std::iter;
///
/// # fn main() -> genco::fmt::Result {
/// let toks = quote! {
///     #(csharp::block_comment(vec!["Foo"]))
///     #(csharp::block_comment(iter::empty::<&str>()))
///     #(csharp::block_comment(vec!["Bar"]))
/// };
///
/// assert_eq!(
///     vec![
///         "/// Foo",
///         "/// Bar",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
pub fn block_comment<T>(comment: T) -> BlockComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    BlockComment(comment)
}

/// Format a doc comment where each line is preceeded by `///`.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let toks = quote! {
///     #(csharp::comment(&["Foo"]))
///     #(csharp::comment(&["Bar"]))
/// };
///
/// assert_eq!(
///     vec![
///         "// Foo",
///         "// Bar",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
pub fn comment<T>(comment: T) -> Comment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    Comment(comment)
}
