//! Specialization for Swift code generation.
//!
//! # String Quoting in Swift
//!
//! Swift uses UTF-8 internally, string quoting is with the exception of escape
//! sequences a one-to-one translation.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: swift::Tokens = quote!("start π 😊 \n \x7f ÿ $ end");
//! assert_eq!("\"start π 😊 \\n \\u{7f} ÿ $ end\"", toks.to_string()?);
//! # Ok(())
//! # }

use crate::fmt;
use crate::lang::{Lang, LangItem};
use crate::tokens::ItemStr;
use std::collections::BTreeSet;
use std::fmt::Write as _;

/// Tokens container specialization for Rust.
pub type Tokens = crate::Tokens<Swift>;

/// Format state for Swift code.
#[derive(Debug, Default)]
pub struct Format {}

/// Configuration for formatting Swift code.
#[derive(Debug, Default)]
pub struct Config {}

impl_dynamic_types! { Swift =>
    pub trait TypeTrait {
        /// Handle imports for the given type.
        fn type_imports(&self, modules: &mut BTreeSet<ItemStr>);
    }

    pub trait Args;
    pub struct Any;
    pub enum AnyRef;

    impl TypeTrait for Type {
        fn type_imports(&self, modules: &mut BTreeSet<ItemStr>) {
            if let Some(module) = &self.module {
                modules.insert(module.clone());
            }
        }
    }

    impl TypeTrait for Map {
        fn type_imports(&self, modules: &mut BTreeSet<ItemStr>) {
            self.key.type_imports(modules);
            self.value.type_imports(modules);
        }
    }

    impl TypeTrait for Array {
        fn type_imports(&self, modules: &mut BTreeSet<ItemStr>) {
            self.inner.type_imports(modules);
        }
    }
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

impl_lang_item! {
    impl LangItem<Swift> for Type {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
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
    key: Any,
    /// Value of the map.
    value: Any,
}

impl_lang_item! {
    impl LangItem<Swift> for Map {
        fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
            out.write_str("[")?;
            self.key.format(out, config, format)?;
            out.write_str(": ")?;
            self.value.format(out, config, format)?;
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
    inner: Any,
}

impl_lang_item! {
    impl LangItem<Swift> for Array {
        fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
            out.write_str("[")?;
            self.inner.format(out, config, format)?;
            out.write_str("]")?;
            Ok(())
        }

        fn as_import(&self) -> Option<&dyn TypeTrait> {
            Some(self)
        }
    }
}

impl Swift {
    fn imports(out: &mut Tokens, tokens: &Tokens) {
        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            import.type_imports(&mut modules);
        }

        if modules.is_empty() {
            return;
        }

        for module in modules {
            let mut s = Tokens::new();

            s.append("import ");
            s.append(module);

            out.append(s);
            out.push();
        }

        out.line();
    }
}

impl Lang for Swift {
    type Config = Config;
    type Format = Format;
    type Import = dyn TypeTrait;

    fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
        // From: https://docs.swift.org/swift-book/LanguageGuide/StringsAndCharacters.html

        for c in input.chars() {
            match c {
                '\0' => out.write_str("\\0")?,
                '\\' => out.write_str("\\\\")?,
                '\t' => out.write_str("\\t")?,
                '\n' => out.write_str("\\n")?,
                '\r' => out.write_str("\\r")?,
                '\'' => out.write_str("\\'")?,
                '"' => out.write_str("\\\"")?,
                c if !c.is_control() => out.write_char(c)?,
                c => {
                    write!(out, "\\u{{{:x}}}", c as u32)?;
                }
            };
        }

        Ok(())
    }

    fn format_file(
        tokens: &Tokens,
        out: &mut fmt::Formatter<'_>,
        config: &Self::Config,
    ) -> fmt::Result {
        let mut imports = Tokens::new();
        Self::imports(&mut imports, tokens);
        let format = Format::default();
        imports.format(out, config, &format)?;
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
/// let toks = quote!(#(swift::imported("Foo", "Debug")));
///
/// assert_eq!(
///     vec![
///         "import Foo",
///         "",
///         "Debug",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
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
/// # fn main() -> genco::fmt::Result {
/// let toks = quote!(#(swift::map(swift::local("String"), swift::imported("Foo", "Debug"))));
///
/// assert_eq!(
///     vec![
///         "import Foo",
///         "",
///         "[String: Debug]",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
pub fn map<K, V>(key: K, value: V) -> Map
where
    K: Into<Any>,
    V: Into<Any>,
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
/// # fn main() -> genco::fmt::Result {
/// let toks = quote!(#(swift::array(swift::imported("Foo", "Debug"))));
///
/// assert_eq!(
///     vec![
///         "import Foo",
///         "",
///         "[Debug]"
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
pub fn array<'a, I>(inner: I) -> Array
where
    I: Into<Any>,
{
    Array {
        inner: inner.into(),
    }
}
