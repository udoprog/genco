//! Specialization for Swift code generation.
//!
//! # Examples
//!
//! String quoting in Swift:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let toks: swift::Tokens = quote!(#("hello \n world".quoted()));
//! assert_eq!("\"hello \\n world\"", toks.to_string().unwrap());
//! ```

use crate::{Formatter, ItemStr, Lang, LangItem};
use std::collections::BTreeSet;
use std::fmt::{self, Write};

/// Tokens container specialization for Rust.
pub type Tokens = crate::Tokens<Swift>;

impl_dynamic_types!(Swift, TypeEnum<'a>, TypeTrait, TypeBox, TypeArgs, {Type, Map, Array});

/// Trait implemented by all types
pub trait TypeTrait: 'static + fmt::Debug + LangItem<Swift> {
    /// Coerce trait into an enum that can be used for type-specific operations
    fn as_enum(&self) -> TypeEnum<'_>;

    /// Handle imports for the given type.
    fn type_imports(&self, modules: &mut BTreeSet<ItemStr>);
}

/// Swift token specialization.
pub struct Swift(());

/// A regular type.
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

impl_lang_item! {
    impl LangItem<Swift> for Type {
        fn format(&self, out: &mut Formatter, _: &mut (), _: usize) -> fmt::Result {
            out.write_str(&self.name)
        }

        fn as_import(&self) -> Option<&dyn TypeTrait> {
            Some(self)
        }
    }
}

/// A map `[<key>: <value>]`.
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

impl_lang_item! {
    impl LangItem<Swift> for Map {
        fn format(&self, out: &mut Formatter, config: &mut (), level: usize) -> fmt::Result {
            out.write_str("[")?;
            self.key.format(out, config, level + 1)?;
            out.write_str(": ")?;
            self.value.format(out, config, level + 1)?;
            out.write_str("]")?;
            Ok(())
        }

        fn as_import(&self) -> Option<&dyn TypeTrait> {
            Some(self)
        }
    }
}

/// An array, `[<inner>]`.
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

impl_lang_item! {
    impl LangItem<Swift> for Array {
        fn format(&self, out: &mut Formatter, config: &mut (), level: usize) -> fmt::Result {
            out.write_str("[")?;
            self.inner.format(out, config, level + 1)?;
            out.write_str("]")?;
            Ok(())
        }

        fn as_import(&self) -> Option<&dyn TypeTrait> {
            Some(self)
        }
    }
}

impl Swift {
    fn imports(tokens: &Tokens) -> Option<Tokens> {
        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            import.type_imports(&mut modules);
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for module in modules {
            let mut s = Tokens::new();

            s.append("import ");
            s.append(module);

            out.append(s);
            out.push();
        }

        Some(out)
    }
}

impl Lang for Swift {
    type Config = ();
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

        if let Some(imports) = Self::imports(&tokens) {
            toks.append(imports);
            toks.line();
        }

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
/// let toks = quote!(#(swift::imported("Foo", "Debug")));
///
/// assert_eq!(
///     vec![
///         "import Foo",
///         "",
///         "Debug",
///     ],
///     toks.to_file_vec().unwrap()
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
/// let toks = quote!(#(swift::map(swift::local("String"), swift::imported("Foo", "Debug"))));
///
/// assert_eq!(
///     vec![
///         "import Foo",
///         "",
///         "[String: Debug]",
///     ],
///     toks.to_file_vec().unwrap()
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
/// let toks = quote!(#(swift::array(swift::imported("Foo", "Debug"))));
///
/// assert_eq!(
///     vec![
///         "import Foo",
///         "",
///         "[Debug]"
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// ```
pub fn array<'a, I>(inner: I) -> Array
where
    I: Into<TypeBox>,
{
    Array {
        inner: inner.into(),
    }
}
