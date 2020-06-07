//! Specialization for Dart code generation.
//!
//! # Examples
//!
//! String quoting in Dart:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let toks: dart::Tokens = quote!(#("hello \n world".quoted()));
//! assert_eq!("\"hello \\n world\"", toks.to_string().unwrap());
//! ```

mod doc_comment;

pub use self::doc_comment::DocComment;

use crate as genco;
use crate::{quote_in, Formatter, ItemStr, Lang, LangItem};
use std::any::Any;
use std::fmt::{self, Write};

/// Tokens container specialization for Dart.
pub type Tokens = crate::Tokens<Dart>;

impl_type_basics!(Dart, TypeEnum<'a>, TypeTrait, TypeBox, TypeArgs, {Type, BuiltIn, Local, Void, Dynamic});

/// Trait implemented by all types
pub trait TypeTrait: 'static + fmt::Debug + LangItem<Dart> {
    /// Coerce trait into an enum that can be used for type-specific operations
    fn as_enum(&self) -> TypeEnum<'_>;
}

static SEP: &'static str = ".";

/// dart:core package.
pub static DART_CORE: &'static str = "dart:core";

/// The type corresponding to `void`.
pub const VOID: Void = Void(());

/// The type corresponding to `dynamic`.
pub const DYNAMIC: Dynamic = Dynamic(());

/// Integer built-in type.
pub const INT: BuiltIn = BuiltIn { name: "int" };

/// Double built-in type.
pub const DOUBLE: BuiltIn = BuiltIn { name: "double" };

/// Boolean built-in type.
pub const BOOL: BuiltIn = BuiltIn { name: "bool" };

impl_modifier! {
    /// A Dart modifier.
    ///
    /// A vector of modifiers have a custom implementation, allowing them to be
    /// formatted with a spacing between them in the language-recommended order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use dart::Modifier::*;
    ///
    /// let toks: dart::Tokens = quote!(#(vec![Final, Async]));
    ///
    /// assert_eq!("async final", toks.to_string().unwrap());
    /// ```
    pub enum Modifier<Dart> {
        /// The `async` modifier.
        Async => "async",
        /// The `final` modifier.
        Final => "final",
    }
}

/// Config data for Dart formatting.
#[derive(Debug, Default)]
pub struct Config {}

/// built-in types.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// assert_eq!("int", quote!(#(dart::INT)).to_string().unwrap());
/// assert_eq!("double", quote!(#(dart::DOUBLE)).to_string().unwrap());
/// assert_eq!("bool", quote!(#(dart::BOOL)).to_string().unwrap());
/// ```
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct BuiltIn {
    /// The built-in type.
    name: &'static str,
}

impl TypeTrait for BuiltIn {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::BuiltIn(self)
    }
}

impl LangItem<Dart> for BuiltIn {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str(self.name)
    }

    fn eq(&self, other: &dyn LangItem<Dart>) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |x| x == self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// a locally defined type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local {
    name: ItemStr,
}

impl TypeTrait for Local {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Local(self)
    }
}

impl LangItem<Dart> for Local {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str(&*self.name)
    }

    fn eq(&self, other: &dyn LangItem<Dart>) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |x| x == self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// the void type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Void(());

impl TypeTrait for Void {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Void(self)
    }
}

impl LangItem<Dart> for Void {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str("void")
    }

    fn eq(&self, other: &dyn LangItem<Dart>) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |x| x == self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The dynamic type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dynamic(());

impl TypeTrait for Dynamic {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Dynamic(self)
    }
}

impl LangItem<Dart> for Dynamic {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str("dynamic")
    }

    fn eq(&self, other: &dyn LangItem<Dart>) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |x| x == self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A custom dart type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Type {
    /// Path to import.
    path: ItemStr,
    /// Name imported.
    name: ItemStr,
    /// Alias of module.
    alias: Option<ItemStr>,
    /// Generic arguments.
    arguments: Vec<TypeBox>,
}

impl Type {
    /// Add an `as` keyword to the import.
    pub fn alias(self, alias: impl Into<ItemStr>) -> Type {
        Self {
            alias: Some(alias.into()),
            ..self
        }
    }

    /// Add arguments to the given variable.
    ///
    /// Only applies to classes, any other will return the same value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// let import = dart::imported("dart:collection", "Map")
    ///     .with_arguments((dart::INT, dart::VOID));
    ///
    /// let toks = quote! {
    ///     #import
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import \"dart:collection\";",
    ///         "",
    ///         "Map<int, void>",
    ///     ],
    ///     toks.to_file_vec().unwrap()
    /// );
    /// ```
    pub fn with_arguments(self, args: impl TypeArgs) -> Type {
        Self {
            arguments: args.into_args(),
            ..self
        }
    }

    /// Convert into raw type.
    pub fn raw(self) -> Type {
        Self {
            arguments: vec![],
            ..self
        }
    }

    /// Check if this type belongs to a core package.
    pub fn is_core(&self) -> bool {
        &*self.path != DART_CORE
    }

    /// Check if type is generic.
    pub fn is_generic(&self) -> bool {
        !self.arguments.is_empty()
    }
}

impl TypeTrait for Type {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Type(self)
    }
}

impl LangItem<Dart> for Type {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        if let Some(alias) = &self.alias {
            out.write_str(alias.as_ref())?;
            out.write_str(SEP)?;
        }

        out.write_str(&*self.name)?;

        if !self.arguments.is_empty() {
            out.write_str("<")?;

            let mut it = self.arguments.iter().peekable();

            while let Some(argument) = it.next() {
                argument.format(out, config, level + 1)?;

                if it.peek().is_some() {
                    out.write_str(", ")?;
                }
            }

            out.write_str(">")?;
        }

        Ok(())
    }

    fn eq(&self, other: &dyn LangItem<Dart>) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |x| x == self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_import(&self) -> Option<&Self> {
        Some(self)
    }
}

/// Language specialization for Dart.
pub struct Dart(());

impl Dart {
    /// Resolve all imports.
    fn imports(input: &Tokens, output: &mut Tokens, _: &mut Config) {
        use crate::ext::QuotedExt as _;
        use std::collections::BTreeSet;

        let mut modules = BTreeSet::new();

        for import in input.walk_imports() {
            if &*import.path == DART_CORE {
                continue;
            }

            modules.insert((import.path.clone(), import.alias.clone()));
        }

        if modules.is_empty() {
            return;
        }

        for (name, alias) in modules {
            if let Some(alias) = alias {
                quote_in!(output => import #(name.quoted()) as #alias;);
            } else {
                quote_in!(output => import #(name.quoted()););
            }

            output.push();
        }

        output.line();
    }
}

impl Lang for Dart {
    type Config = Config;
    type Import = Type;

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

    fn write_file(
        tokens: Tokens,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks: Tokens = Tokens::new();
        Self::imports(&tokens, &mut toks, config);
        toks.extend(tokens);
        toks.format(out, config, level)
    }
}

/// Setup an imported element.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// let a = dart::imported("package:http/http.dart", "A");
/// let b = dart::imported("package:http/http.dart", "B");
/// let c = dart::imported("package:http/http.dart", "C").alias("h2");
/// let d = dart::imported("../http.dart", "D");
///
/// let toks = quote! {
///     #a
///     #b
///     #c
///     #d
/// };
///
/// let expected = vec![
///     "import \"../http.dart\";",
///     "import \"package:http/http.dart\";",
///     "import \"package:http/http.dart\" as h2;",
///     "",
///     "A",
///     "B",
///     "h2.C",
///     "D",
/// ];
///
/// assert_eq!(expected, toks.to_file_vec().unwrap());
/// ```
pub fn imported<P: Into<ItemStr>, N: Into<ItemStr>>(path: P, name: N) -> Type {
    Type {
        path: path.into(),
        alias: None,
        name: name.into(),
        arguments: Vec::new(),
    }
}

/// Setup a local element.
pub fn local<N: Into<ItemStr>>(name: N) -> Local {
    Local { name: name.into() }
}

/// Format a doc comment where each line is preceeded by `///`.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// use std::iter;
///
/// let toks = quote! {
///     #(dart::doc_comment(vec!["Foo"]))
///     #(dart::doc_comment(iter::empty::<&str>()))
///     #(dart::doc_comment(vec!["Bar"]))
/// };
///
/// assert_eq!(
///     vec![
///         "/// Foo",
///         "/// Bar",
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// ```
pub fn doc_comment<T>(comment: T) -> DocComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    DocComment(comment)
}
