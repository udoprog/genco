//! Specialization for Go code generation.
//!
//! # Examples
//!
//! Basic example:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let toks: js::Tokens = quote! {
//!     function foo(v) {
//!         return v + ", World";
//!     }
//!
//!     foo("Hello");
//! };
//!
//! assert_eq!(
//!     vec![
//!         "function foo(v) {",
//!         "    return v + \", World\";",
//!         "}",
//!         "",
//!         "foo(\"Hello\");",
//!     ],
//!     toks.to_file_vec().unwrap()
//! );
//! ```
//!
//! String quoting in JavaScript:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let toks: go::Tokens = quote!(#("hello \n world".quoted()));
//! assert_eq!("\"hello \\n world\"", toks.to_string().unwrap());
//! ```

use crate as genco;
use crate::{quote_in, Formatter, ItemStr, Lang, LangItem};
use std::collections::BTreeSet;
use std::fmt::{self, Write};

/// Tokens container specialization for Go.
pub type Tokens = crate::Tokens<Go>;

impl_type_basics!(Go, TypeEnum<'a>, TypeTrait, TypeBox, TypeArgs, {Type, Map, Array, Interface});

/// Trait implemented by all types
pub trait TypeTrait: 'static + fmt::Debug + LangItem<Go> {
    /// Coerce trait into an enum that can be used for type-specific operations
    fn as_enum(&self) -> TypeEnum<'_>;

    /// Handle imports for the given type.
    fn type_imports(&self, _: &mut BTreeSet<ItemStr>) {}
}

/// The interface type `interface{}`.
pub const INTERFACE: Interface = Interface(());

const SEP: &str = ".";

/// A Go type.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type {
    /// Module of the imported name.
    module: Option<ItemStr>,
    /// Name imported.
    name: ItemStr,
}

impl TypeTrait for Type {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Type(self)
    }

    fn type_imports(&self, modules: &mut BTreeSet<ItemStr>) {
        if let Some(module) = &self.module {
            modules.insert(module.clone());
        }
    }
}

impl LangItem<Go> for Type {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        if let Some(module) = self.module.as_ref().and_then(|m| m.split("/").last()) {
            out.write_str(module)?;
            out.write_str(SEP)?;
        }

        out.write_str(&self.name)?;
        Ok(())
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// A map `map[<key>]<value>`.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Map {
    /// Key of the map.
    key: TypeBox,
    /// Value of the map.
    value: TypeBox,
}

impl TypeTrait for Map {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Map(self)
    }

    fn type_imports(&self, modules: &mut BTreeSet<ItemStr>) {
        self.key.type_imports(modules);
        self.value.type_imports(modules);
    }
}

impl LangItem<Go> for Map {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        out.write_str("map[")?;
        self.key.format(out, config, level + 1)?;
        out.write_str("]")?;
        self.value.format(out, config, level + 1)?;
        Ok(())
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// An array `[]<inner>`.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Array {
    /// Inner value of the array.
    inner: TypeBox,
}

impl TypeTrait for Array {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Array(self)
    }

    fn type_imports(&self, modules: &mut BTreeSet<ItemStr>) {
        self.inner.type_imports(modules);
    }
}

impl LangItem<Go> for Array {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        out.write_str("[")?;
        out.write_str("]")?;
        self.inner.format(out, config, level + 1)?;
        Ok(())
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// The interface type `interface{}`.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Interface(());

impl TypeTrait for Interface {
    fn as_enum(&self) -> TypeEnum<'_> {
        TypeEnum::Interface(self)
    }
}

impl LangItem<Go> for Interface {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str("interface{}")
    }

    fn as_import(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }
}

/// Config data for Go.
#[derive(Debug, Default)]
pub struct Config {
    package: Option<ItemStr>,
}

impl Config {
    /// Configure the specified package.
    pub fn with_package<P: Into<ItemStr>>(self, package: P) -> Self {
        Self {
            package: Some(package.into()),
            ..self
        }
    }
}

/// Language specialization for Go.
pub struct Go(());

impl Go {
    fn imports(tokens: &Tokens) -> Option<Tokens> {
        use crate::ext::QuotedExt as _;

        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            import.type_imports(&mut modules);
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for module in modules {
            quote_in!(out => import #(module.quoted()));
            out.push();
        }

        Some(out)
    }
}

impl Lang for Go {
    type Config = Config;
    type Import = dyn TypeTrait;

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
        tokens: Tokens,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks = Tokens::new();

        if let Some(package) = &config.package {
            quote_in!(toks => package #package);
            toks.line();
        }

        if let Some(imports) = Self::imports(&tokens) {
            toks.append(imports);
            toks.line();
        }

        toks.line();
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
/// let ty = go::imported("foo", "Debug");
///
/// let toks = quote! {
///     #ty
/// };
///
/// assert_eq!(
///     vec![
///        "package foo",
///        "",
///        "import \"foo\"",
///        "",
///        "foo.Debug",
///     ],
///     toks.to_file_vec_with(
///         go::Config::default().with_package("foo"),
///         FormatterConfig::from_lang::<Go>()
///     )
///     .unwrap()
/// );
/// ```
pub fn imported<M, N>(module: M, name: N) -> Type
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Type {
        module: Some(module.into()),
        name: name.into(),
    }
}

/// Setup a local element.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// let toks = quote!(#(go::local("MyType")));
/// assert_eq!(vec!["MyType"], toks.to_file_vec().unwrap());
/// ```
pub fn local<N>(name: N) -> Type
where
    N: Into<ItemStr>,
{
    Type {
        module: None,
        name: name.into(),
    }
}

/// Setup a map.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// let ty = go::map(go::imported("foo", "Debug"), go::INTERFACE);
///
/// let toks = quote! {
///     #ty
/// };
///
/// assert_eq!(
///     vec![
///         "package foo",
///         "",
///         "import \"foo\"",
///         "",
///         "map[foo.Debug]interface{}",
///     ],
///     toks.to_file_vec_with(
///         go::Config::default().with_package("foo"),
///         FormatterConfig::from_lang::<Go>()
///     )
///     .unwrap()
/// );
/// ```
pub fn map<K, V>(key: K, value: V) -> Map
where
    K: Into<TypeBox>,
    V: Into<TypeBox>,
{
    Map {
        key: key.into(),
        value: value.into(),
    }
}

/// Setup an array.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// let import = go::array(go::imported("foo", "Debug"));
///
/// let toks = quote!(#import);
///
/// assert_eq!(
///     vec![
///         "package foo",
///         "",
///         "import \"foo\"",
///         "",
///         "[]foo.Debug",
///     ],
///     toks.to_file_vec_with(
///         go::Config::default().with_package("foo"),
///         FormatterConfig::from_lang::<Go>()
///     )
///     .unwrap()
/// );
/// ```
pub fn array<I>(inner: I) -> Array
where
    I: Into<TypeBox>,
{
    Array {
        inner: inner.into(),
    }
}
