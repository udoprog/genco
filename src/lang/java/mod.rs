//! Specialization for Java code generation.
//!
//! # String Quoting in Java
//!
//! Since Java uses UTF-16 internally, string quoting for high unicode
//! characters is done through surrogate pairs, as seen with the 😊 below.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: java::Tokens = quote!("start π 😊 \n \x7f end");
//! assert_eq!("\"start \\u03c0 \\ud83d\\ude0a \\n \\u007f end\"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```

mod block_comment;

pub use self::block_comment::BlockComment;

use crate as genco;
use crate::fmt;
use crate::lang::{Lang, LangItem};
use crate::tokens::ItemStr;
use crate::{quote, quote_in};
use std::collections::{BTreeSet, HashMap};

/// Tokens container specialized for Java.
pub type Tokens = crate::Tokens<Java>;

impl_dynamic_types! { Java =>
    trait TypeTrait {
        /// Get package type belongs to.
        fn name(&self) -> &str;

        /// Get package type belongs to.
        fn package(&self) -> Option<&str> {
            None
        }

        /// Get generic arguments associated with type.
        fn arguments(&self) -> Option<&[Any]> {
            None
        }

        /// Process which kinds of imports to deal with.
        fn type_imports(&self, _: &mut BTreeSet<(ItemStr, ItemStr)>) {}

        /// Java-specific interior formatting.
        fn java_format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format, level: usize) -> fmt::Result;
    }

    Primitive {
        impl TypeTrait {
            fn name(&self) -> &str {
                self.primitive
            }

            fn package(&self) -> Option<&str> {
                Some(JAVA_LANG)
            }

            fn java_format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format, level: usize) -> fmt::Result {
                if level > 0 {
                    out.write_str(self.boxed)
                } else {
                    out.write_str(self.primitive)
                }
            }
        }

        impl LangItem {
            fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
                self.java_format(out, config, format, 0)
            }
        }
    }

    Void {
        impl TypeTrait {
            fn name(&self) -> &str {
                "void"
            }

            fn java_format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format, level: usize) -> fmt::Result {
                if level > 0 {
                    out.write_str("Void")
                } else {
                    out.write_str("void")
                }
            }
        }

        impl LangItem {
            fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
                self.java_format(out, config, format, 0)
            }
        }
    }

    Type {
        impl TypeTrait {
            fn name(&self) -> &str {
                &*self.name
            }

            fn package(&self) -> Option<&str> {
                Some(&*self.package)
            }

            fn arguments(&self) -> Option<&[Any]> {
                Some(&self.arguments)
            }

            fn type_imports(&self, modules: &mut BTreeSet<(ItemStr, ItemStr)>) {
                for argument in &self.arguments {
                    if let AnyRef::Type(ty) = argument.as_enum() {
                        ty.type_imports(modules);
                    }
                }

                modules.insert((self.package.clone(), self.name.clone()));
            }

            fn java_format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format, level: usize) -> fmt::Result {
                {
                    let file_package = config.package.as_ref().map(|p| p.as_ref());
                    let imported = format.imported.get(self.name.as_ref()).map(String::as_str);
                    let pkg = Some(self.package.as_ref());

                    if self.package.as_ref() != JAVA_LANG && imported != pkg && file_package != pkg {
                        out.write_str(self.package.as_ref())?;
                        out.write_str(SEP)?;
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
                        argument.java_format(out, config, format, level + 1)?;

                        if it.peek().is_some() {
                            out.write_str(", ")?;
                        }
                    }

                    out.write_str(">")?;
                }

                Ok(())
            }
        }

        impl LangItem {
            fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
                self.java_format(out, config, format, 0)
            }

            fn as_import(&self) -> Option<&dyn TypeTrait> {
                Some(self)
            }
        }
    }

    Optional {
        impl TypeTrait {
            fn name(&self) -> &str {
                self.value.name()
            }

            fn package(&self) -> Option<&str> {
                self.value.package()
            }

            fn arguments(&self) -> Option<&[Any]> {
                self.value.arguments()
            }

            fn type_imports(&self, modules: &mut BTreeSet<(ItemStr, ItemStr)>) {
                self.value.type_imports(modules);
            }

            fn java_format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format, level: usize) -> fmt::Result {
                self.field.java_format(out, config, format, level)
            }
        }

        impl LangItem {
            fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
                self.java_format(out, config, format, 0)
            }

            fn as_import(&self) -> Option<&dyn TypeTrait> {
                Some(self)
            }
        }
    }

    Local {
        impl TypeTrait {
            fn name(&self) -> &str {
                &*self.name
            }

            fn java_format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format, _: usize) -> fmt::Result {
                out.write_str(&*self.name)
            }
        }

        impl LangItem {
            fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
                self.java_format(out, config, format, 0)
            }

            fn as_import(&self) -> Option<&dyn TypeTrait> {
                Some(self)
            }
        }
    }
}

const JAVA_LANG: &'static str = "java.lang";
const SEP: &'static str = ".";

/// Short primitive type.
pub const SHORT: Primitive = Primitive {
    primitive: "short",
    boxed: "Short",
};

/// Integer primitive type.
pub const INTEGER: Primitive = Primitive {
    primitive: "int",
    boxed: "Integer",
};

/// Long primitive type.
pub const LONG: Primitive = Primitive {
    primitive: "long",
    boxed: "Long",
};

/// Float primitive type.
pub const FLOAT: Primitive = Primitive {
    primitive: "float",
    boxed: "Float",
};

/// Double primitive type.
pub const DOUBLE: Primitive = Primitive {
    primitive: "double",
    boxed: "Double",
};

/// Char primitive type.
pub const CHAR: Primitive = Primitive {
    primitive: "char",
    boxed: "Character",
};

/// Boolean primitive type.
pub const BOOLEAN: Primitive = Primitive {
    primitive: "boolean",
    boxed: "Boolean",
};

/// Byte primitive type.
pub const BYTE: Primitive = Primitive {
    primitive: "byte",
    boxed: "Byte",
};

/// Void type.
pub const VOID: Void = Void(());

/// Formtat state for Java.
#[derive(Debug, Default)]
pub struct Format {
    /// Types which has been imported into the local namespace.
    imported: HashMap<String, String>,
}

/// Configuration for Java.
#[derive(Debug, Default)]
pub struct Config {
    /// Package to use.
    package: Option<ItemStr>,
}

impl Config {
    /// Configure package to use.
    pub fn with_package(self, package: impl Into<ItemStr>) -> Self {
        Self {
            package: Some(package.into()),
            ..self
        }
    }
}

impl_modifier! {
    /// A Java modifier.
    ///
    /// A vector of modifiers have a custom implementation, allowing them to be
    /// formatted with a spacing between them in the language-recommended order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use java::Modifier::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let toks: java::Tokens = quote!(#(vec![Public, Final, Static]));
    ///
    /// assert_eq!("public static final", toks.to_string()?);
    /// # Ok(())
    /// # }
    /// ```
    pub enum Modifier<Java> {
        /// The `default` modifier.
        Default => "default",
        /// The `public` modifier.
        Public => "public",
        /// The `protected` modifier.
        Protected => "protected",
        /// The `private` modifier.
        Private => "private",
        /// The `abstract` modifier.
        Abstract => "abstract",
        /// The `static` modifier.
        Static => "static",
        /// The `final` modifier.
        Final => "final",
        /// The `native` modifier.
        Native => "native",
    }
}

/// A class.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type {
    /// Package of the class.
    package: ItemStr,
    /// Name  of class.
    name: ItemStr,
    /// Path of class when nested.
    path: Vec<ItemStr>,
    /// Arguments of the class.
    arguments: Vec<Any>,
}

impl Type {
    /// Extend the type with a nested path.
    ///
    /// This discards any arguments associated with it.
    pub fn path<P: Into<ItemStr>>(self, part: P) -> Self {
        let mut path = self.path;
        path.push(part.into());

        Self {
            package: self.package,
            name: self.name,
            path: path,
            arguments: vec![],
        }
    }

    /// Add arguments to the given variable.
    ///
    /// Only applies to classes, any other will return the same value.
    pub fn with_arguments(self, args: impl Args) -> Self {
        Self {
            package: self.package,
            name: self.name,
            path: self.path,
            arguments: args.into_args(),
        }
    }

    /// Get the raw type.
    ///
    /// A raw type is one without generic arguments.
    pub fn as_raw(self) -> Self {
        Self {
            package: self.package,
            name: self.name,
            path: self.path,
            arguments: vec![],
        }
    }

    /// Check if type is generic.
    pub fn is_generic(&self) -> bool {
        !self.arguments.is_empty()
    }
}

/// The void type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Void(());
/// A primitive type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Primitive {
    /// The boxed variant of the primitive type.
    boxed: &'static str,
    /// The primitive-primitive type.
    primitive: &'static str,
}

impl Primitive {
    /// Get a boxed version of a primitive type.
    pub const fn into_boxed(self) -> Type {
        Type {
            package: ItemStr::Static(JAVA_LANG),
            name: ItemStr::Static(self.boxed),
            path: vec![],
            arguments: vec![],
        }
    }
}

/// A local name with no specific qualification.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local {
    /// Name of class.
    name: ItemStr,
}

/// An optional type.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Optional {
    /// The type that is optional.
    pub value: Any,
    /// The complete optional field type, including wrapper.
    pub field: Any,
}

impl Optional {
    /// Get the field type (includes optionality).
    pub fn as_field(self) -> Any {
        self.field.clone()
    }

    /// Get the value type (strips optionality).
    pub fn as_value(self) -> Any {
        self.value.clone()
    }
}

/// Language specialization for Java.
pub struct Java(());

impl Java {
    fn imports(
        out: &mut Tokens,
        tokens: &Tokens,
        config: &Config,
        imported: &mut HashMap<String, String>,
    ) {
        let mut modules = BTreeSet::new();

        let file_package = config.package.as_ref().map(|p| p.as_ref());

        for import in tokens.walk_imports() {
            import.type_imports(&mut modules);
        }

        if modules.is_empty() {
            return;
        }

        for (package, name) in modules {
            if imported.contains_key(&*name) {
                continue;
            }

            if &*package == JAVA_LANG {
                continue;
            }

            if Some(&*package) == file_package.as_deref() {
                continue;
            }

            out.append(quote!(import #(package.clone())#(SEP)#(name.clone());));
            out.push();

            imported.insert(name.to_string(), package.to_string());
        }

        out.line();
    }
}

impl Lang for Java {
    type Config = Config;
    type Format = Format;
    type Import = dyn TypeTrait;

    fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
        // From: https://docs.oracle.com/javase/tutorial/java/data/characters.html
        use std::fmt::Write as _;

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
                ' ' => out.write_char(' ')?,
                c if c.is_ascii() && !c.is_control() => out.write_char(c)?,
                c => {
                    for c in c.encode_utf16(&mut [0u16; 2]) {
                        write!(out, "\\u{:04x}", c)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn format_file(
        tokens: &Tokens,
        out: &mut fmt::Formatter<'_>,
        config: &Self::Config,
    ) -> fmt::Result {
        let mut header = Tokens::new();

        if let Some(ref package) = config.package {
            quote_in!(header => package #package;);
            header.line();
        }

        let mut format = Format::default();
        Self::imports(&mut header, tokens, config, &mut format.imported);
        header.format(out, config, &format)?;
        tokens.format(out, config, &format)?;
        Ok(())
    }
}

/// Setup an imported element.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let integer = java::imported("java.lang", "Integer");
/// let a = java::imported("java.io", "A");
/// let b = java::imported("java.io", "B");
/// let ob = java::imported("java.util", "B");
/// let ob_a = ob.clone().with_arguments(a.clone());
///
/// let toks = quote! {
///     #integer
///     #a
///     #b
///     #ob
///     #ob_a
/// };
///
/// assert_eq!(
///     vec![
///         "import java.io.A;",
///         "import java.io.B;",
///         "",
///         "Integer",
///         "A",
///         "B",
///         "java.util.B",
///         "java.util.B<A>"
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
pub fn imported<P: Into<ItemStr>, N: Into<ItemStr>>(package: P, name: N) -> Type {
    Type {
        package: package.into(),
        name: name.into(),
        path: vec![],
        arguments: vec![],
    }
}

/// Setup a local element from borrowed components.
pub fn local<N: Into<ItemStr>>(name: N) -> Local {
    Local { name: name.into() }
}

/// Setup an optional type.
pub fn optional<I: Into<Any>, F: Into<Any>>(value: I, field: F) -> Optional {
    Optional {
        value: value.into(),
        field: field.into(),
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
///     #(java::block_comment(vec!["first line", "second line"]))
///     #(java::block_comment(iter::empty::<&str>()))
///     #(java::block_comment(vec!["third line"]))
/// };
///
/// assert_eq!(
///     vec![
///         "/**",
///         " * first line",
///         " * second line",
///         " */",
///         "/**",
///         " * third line",
///         " */",
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
