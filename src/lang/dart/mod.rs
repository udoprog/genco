//! Specialization for Dart code generation.
//!
//! # Examples
//!
//! String quoting in Dart:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: dart::Tokens = quote!(#("hello \n world".quoted()));
//! assert_eq!("\"hello \\n world\"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```

mod doc_comment;

pub use self::doc_comment::DocComment;

use crate as genco;
use crate::fmt;
use crate::{quote_in, ItemStr, Lang, LangItem};
use std::fmt::Write as _;

/// Tokens container specialization for Dart.
pub type Tokens = crate::Tokens<Dart>;

impl_dynamic_types! { Dart =>
    pub trait TypeTrait {}
    pub trait Args;
    pub struct Any;
    pub enum AnyRef;

    impl TypeTrait for Type {}
    impl TypeTrait for BuiltIn {}
    impl TypeTrait for Local {}
    impl TypeTrait for Void {}
    impl TypeTrait for Dynamic {}
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
    /// # fn main() -> genco::fmt::Result {
    /// let toks: dart::Tokens = quote!(#(vec![Final, Async]));
    ///
    /// assert_eq!("async final", toks.to_string()?);
    /// # Ok(())
    /// # }
    /// ```
    pub enum Modifier<Dart> {
        /// The `async` modifier.
        Async => "async",
        /// The `final` modifier.
        Final => "final",
    }
}

/// Format state for Dart.
#[derive(Debug, Default)]
pub struct Format {}

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
/// # fn main() -> genco::fmt::Result {
/// assert_eq!("int", quote!(#(dart::INT)).to_string()?);
/// assert_eq!("double", quote!(#(dart::DOUBLE)).to_string()?);
/// assert_eq!("bool", quote!(#(dart::BOOL)).to_string()?);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct BuiltIn {
    /// The built-in type.
    name: &'static str,
}

impl_lang_item! {
    impl LangItem<Dart> for BuiltIn {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str(self.name)
        }
    }
}

/// a locally defined type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Local {
    name: ItemStr,
}

impl_lang_item! {
    impl LangItem<Dart> for Local {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str(&*self.name)
        }
    }
}

/// the void type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Void(());

impl_lang_item! {
    impl LangItem<Dart> for Void {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str("void")
        }
    }
}

/// The dynamic type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dynamic(());

impl_lang_item! {
    impl LangItem<Dart> for Dynamic {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str("dynamic")
        }
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
    arguments: Vec<Any>,
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
    /// # fn main() -> genco::fmt::Result {
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
    ///     toks.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_arguments(self, args: impl Args) -> Type {
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

impl_lang_item! {
    impl LangItem<Dart> for Type {
        fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
            if let Some(alias) = &self.alias {
                out.write_str(alias.as_ref())?;
                out.write_str(SEP)?;
            }

            out.write_str(&*self.name)?;

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

        fn as_import(&self) -> Option<&Self> {
            Some(self)
        }
    }
}

/// Language specialization for Dart.
pub struct Dart(());

impl Dart {
    /// Resolve all imports.
    fn imports(out: &mut Tokens, input: &Tokens, _: &Config) {
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
                quote_in!(*out => import #(name.quoted()) as #alias;);
            } else {
                quote_in!(*out => import #(name.quoted()););
            }

            out.push();
        }

        out.line();
    }
}

impl Lang for Dart {
    type Config = Config;
    type Format = Format;
    type Import = Type;

    fn quote_string(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
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

    fn format_file(
        tokens: &Tokens,
        out: &mut fmt::Formatter<'_>,
        config: &Self::Config,
    ) -> fmt::Result {
        let mut imports: Tokens = Tokens::new();
        Self::imports(&mut imports, tokens, config);
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
/// assert_eq!(expected, toks.to_file_vec()?);
/// # Ok(())
/// # }
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
/// use std::iter;
///
/// # fn main() -> genco::fmt::Result {
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
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
pub fn doc_comment<T>(comment: T) -> DocComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    DocComment(comment)
}
