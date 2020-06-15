//! Specialization for JavaScript code generation.
//!
//! # Examples
//!
//! Basic example:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
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
//!     toks.to_file_vec()?
//! );
//! # Ok(())
//! # }
//! ```
//!
//! String quoting in JavaScript:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: js::Tokens = quote!("hello \n world");
//! assert_eq!("\"hello \\n world\"", toks.to_string()?);
//!
//! let toks: js::Tokens = quote!(#(quoted("hello \n world")));
//! assert_eq!("\"hello \\n world\"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```

use crate::fmt;
use crate::lang::Lang;
use crate::tokens::ItemStr;
use relative_path::RelativePathBuf;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// Tokens container specialization for Rust.
pub type Tokens = crate::Tokens<JavaScript>;

impl_dynamic_types! {
    /// JavaScript language specialization.
    pub JavaScript
    =>
    trait TypeTrait {}

    Import {
        impl TypeTrait {}

        impl LangItem {
            fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
                if let Some(alias) = &self.alias {
                    out.write_str(alias)?;
                } else {
                    out.write_str(&self.name)?;
                }

                Ok(())
            }

            fn as_import(&self) -> Option<&dyn TypeTrait> {
                Some(self)
            }
        }
    }

    ImportDefault {
        impl TypeTrait {}

        impl LangItem {
            fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
                out.write_str(&self.name)
            }

            fn as_import(&self) -> Option<&dyn TypeTrait> {
                Some(self)
            }
        }
    }
}

/// Format state for JavaScript.
#[derive(Debug, Default)]
pub struct Format {}

/// Configuration for JavaScript.
#[derive(Debug, Default)]
pub struct Config {
    module_path: Option<RelativePathBuf>,
}

impl Config {
    /// Configure the path to the current module being renderer.
    ///
    /// This setting will determine what path imports are renderer relative
    /// towards. So importing a module from `"foo/bar.js"`, and setting this to
    /// `"foo/baz.js"` will cause the import to be rendered relatively as
    /// `"../bar.js"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use genco::fmt;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let foo1 = js::import(js::Module::Path("foo/bar.js".into()), "Foo1");
    /// let foo2 = js::import(js::Module::Path("foo/bar.js".into()), "Foo2");
    /// let react = js::import_default("react", "React");
    ///
    /// let toks: js::Tokens = quote! {
    ///     #foo1
    ///     #foo2
    ///     #react
    /// };
    ///
    /// let mut w = fmt::VecWriter::new();
    ///
    /// let config = js::Config::default().with_module_path("foo/baz.js");
    /// let fmt = fmt::Config::from_lang::<JavaScript>();
    ///
    /// toks.format_file(&mut w.as_formatter(fmt), &config)?;
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import {Foo1, Foo2} from \"../bar.js\";",
    ///         "import React from \"react\";",
    ///         "",
    ///         "Foo1",
    ///         "Foo2",
    ///         "React"
    ///     ],
    ///     w.into_vec()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_module_path<M>(self, module_path: M) -> Self
    where
        M: Into<RelativePathBuf>,
    {
        Self {
            module_path: Some(module_path.into()),
            ..self
        }
    }
}

/// An imported JavaScript item `import {foo} from "module.js"`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// Module of the imported name.
    module: Module,
    /// Name imported.
    name: ItemStr,
    /// Alias of an imported item.
    ///
    /// If this is set, you'll get an import like:
    ///
    /// ```text
    /// import {<name> as <alias>} from <module>
    /// ```
    alias: Option<ItemStr>,
}

impl Import {
    /// Alias of an imported item.
    ///
    /// If this is set, you'll get an import like:
    ///
    /// ```text
    /// import {<name> as <alias>} from <module>
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let a = js::import("collections", "vec");
    /// let b = js::import("collections", "vec").with_alias("list");
    ///
    /// let toks = quote! {
    ///     #a
    ///     #b
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import {vec, vec as list} from \"collections\";",
    ///         "",
    ///         "vec",
    ///         "list",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_alias<N>(self, alias: N) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            alias: Some(alias.into()),
            ..self
        }
    }
}

/// A module being imported.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Module {
    /// A module imported from a specific path.
    ///
    /// The path will be relativized according to the module specified in the
    /// [Config::with_module_path].
    Path(RelativePathBuf),
    /// A globally imported module.
    Global(ItemStr),
}

impl<'a> From<&'a str> for Module {
    fn from(value: &'a str) -> Self {
        Self::Global(value.into())
    }
}

impl From<String> for Module {
    fn from(value: String) -> Self {
        Self::Global(value.into())
    }
}

/// The import of a default JavaScript item from a module
/// `import foo from "module.js"`.
///
/// Created through the [import_default()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ImportDefault {
    /// Module of the imported name.
    module: Module,
    /// Name imported.
    name: ItemStr,
}

impl JavaScript {
    /// Translate imports into the necessary tokens.
    fn imports(out: &mut Tokens, tokens: &Tokens, config: &Config) {
        use crate as genco;
        use crate::prelude::*;

        let mut modules = BTreeMap::<&Module, ResolvedModule<'_>>::new();

        for import in tokens.walk_imports() {
            match import.as_enum() {
                AnyRef::Import(this) => {
                    let module = modules.entry(&this.module).or_default();

                    module.set.insert(match &this.alias {
                        None => ImportedElement::Plain(&this.name),
                        Some(alias) => ImportedElement::Aliased(&this.name, alias),
                    });
                }
                AnyRef::ImportDefault(this) => {
                    let module = modules.entry(&this.module).or_default();
                    module.default_import = Some(&this.name);
                }
            }
        }

        if modules.is_empty() {
            return;
        }

        for (name, module) in modules {
            out.push();
            quote_in! { *out =>
                import #(ref tokens => {
                    if let Some(default) = module.default_import {
                        tokens.append(ItemStr::from(default));

                        if !module.set.is_empty() {
                            tokens.append(",");
                            tokens.space();
                        }
                    }

                    if !module.set.is_empty() {
                        tokens.append("{");

                        let mut it = module.set.iter().peekable();

                        while let Some(el) = it.next() {
                            match *el {
                                ImportedElement::Plain(name) => {
                                    tokens.append(name);
                                },
                                ImportedElement::Aliased(name, alias) => {
                                    quote_in!(*tokens => #name as #alias);
                                }
                            }

                            if it.peek().is_some() {
                                tokens.append(",");
                                tokens.space();
                            }
                        }

                        tokens.append("}");
                    }
                }) from #(match (&config.module_path, name) {
                    (_, Module::Global(from)) => #(quoted(from)),
                    (None, Module::Path(path)) => #(quoted(path.as_str())),
                    (Some(module_path), Module::Path(path)) => #(quoted(module_path.relative(path).as_str())),
                });
            };
        }

        out.line();

        #[derive(Default)]
        struct ResolvedModule<'a> {
            default_import: Option<&'a ItemStr>,
            set: BTreeSet<ImportedElement<'a>>,
        }

        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
        enum ImportedElement<'a> {
            Plain(&'a ItemStr),
            Aliased(&'a ItemStr, &'a ItemStr),
        }
    }
}

impl Lang for JavaScript {
    type Config = Config;
    type Format = Format;
    type Import = dyn TypeTrait;

    /// Start a string quote.
    fn open_quote(
        out: &mut fmt::Formatter<'_>,
        _config: &Self::Config,
        _format: &Self::Format,
        has_eval: bool,
    ) -> fmt::Result {
        use std::fmt::Write as _;

        if has_eval {
            out.write_char('`')?;
        } else {
            out.write_char('"')?;
        }

        Ok(())
    }

    /// End a string quote.
    fn close_quote(
        out: &mut fmt::Formatter<'_>,
        _config: &Self::Config,
        _format: &Self::Format,
        has_eval: bool,
    ) -> fmt::Result {
        use std::fmt::Write as _;

        if has_eval {
            out.write_char('`')?;
        } else {
            out.write_char('"')?;
        }

        Ok(())
    }

    fn start_string_eval(
        out: &mut fmt::Formatter<'_>,
        _config: &Self::Config,
        _format: &Self::Format,
    ) -> fmt::Result {
        out.write_str("${")?;
        Ok(())
    }

    fn end_string_eval(
        out: &mut fmt::Formatter<'_>,
        _config: &Self::Config,
        _format: &Self::Format,
    ) -> fmt::Result {
        out.write_char('}')?;
        Ok(())
    }

    fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
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
        Self::imports(&mut imports, tokens, config);
        let format = Format::default();
        imports.format(out, config, &format)?;
        tokens.format(out, config, &format)?;
        Ok(())
    }
}

/// Import an element from a module
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let a = js::import("collections", "vec");
/// let b = js::import("collections", "vec").with_alias("list");
///
/// let toks = quote! {
///     #a
///     #b
/// };
///
/// assert_eq!(
///     vec![
///         "import {vec, vec as list} from \"collections\";",
///         "",
///         "vec",
///         "list",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
pub fn import<M, N>(module: M, name: N) -> Import
where
    M: Into<Module>,
    N: Into<ItemStr>,
{
    Import {
        module: module.into(),
        name: name.into(),
        alias: None,
    }
}

/// Import the default element from the specified module.
///
/// Note that the default element may only be aliased once, so multiple aliases
/// will cause an error.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let a = js::import_default("collections", "defaultVec");
/// let b = js::import("collections", "vec");
/// let c = js::import("collections", "vec").with_alias("list");
///
/// let toks = quote! {
///     #a
///     #b
///     #c
/// };
///
/// assert_eq!(
///     vec![
///         "import defaultVec, {vec, vec as list} from \"collections\";",
///         "",
///         "defaultVec",
///         "vec",
///         "list",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok(())
/// # }
/// ```
pub fn import_default<M, N>(module: M, name: N) -> ImportDefault
where
    M: Into<Module>,
    N: Into<ItemStr>,
{
    ImportDefault {
        module: module.into(),
        name: name.into(),
    }
}
