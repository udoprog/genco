//! Specialization for Python code generation.
//!
//! # Examples
//!
//! String quoting in Python:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: python::Tokens = quote!("hello \n world");
//! assert_eq!("\"hello \\n world\"", toks.to_string()?);
//!
//! let toks: python::Tokens = quote!($(quoted("hello \n world")));
//! assert_eq!("\"hello \\n world\"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```

use core::fmt::Write as _;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate as genco;
use crate::fmt;
use crate::tokens::ItemStr;
use crate::{quote, quote_in};

/// Tokens container specialization for Python.
pub type Tokens = crate::Tokens<Python>;

impl_lang! {
    /// Language specialization for Python.
    pub Python {
        type Config = Config;
        type Format = Format;
        type Item = Any;

        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
            // From: https://docs.python.org/3/reference/lexical_analysis.html#string-and-bytes-literals
            super::c_family_write_quoted(out, input)
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

    Import {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            if let TypeModule::Qualified { module, alias }  = &self.module {
                out.write_str(alias.as_ref().unwrap_or(module))?;
                out.write_str(SEP)?;
            }

            let name = match &self.alias {
                Some(alias) => alias,
                None => &self.name,
            };

            out.write_str(name)?;
            Ok(())
        }
    }

    ImportModule {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            let module = match &self.alias {
                Some(alias) => alias,
                None => &self.module,
            };

            out.write_str(module)?;
            Ok(())
        }
    }
}

/// Formatting state for python.
#[derive(Debug, Default)]
pub struct Format {}
/// Configuration for python.
#[derive(Debug, Default)]
pub struct Config {}

static SEP: &str = ".";

#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
enum TypeModule {
    Unqualified {
        /// Name of imported module.
        module: ItemStr,
    },
    Qualified {
        /// Name of imported module.
        module: ItemStr,
        /// Alias of imported module.
        alias: Option<ItemStr>,
    },
}

impl TypeModule {
    fn qualified(self) -> Self {
        match self {
            Self::Unqualified { module } => Self::Qualified {
                module,
                alias: None,
            },
            other => other,
        }
    }

    fn with_alias<T>(self, alias: T) -> Self
    where
        T: Into<ItemStr>,
    {
        match self {
            Self::Qualified { module, .. } | Self::Unqualified { module } => Self::Qualified {
                module,
                alias: Some(alias.into()),
            },
        }
    }
}

/// The import of a Python name `from module import foo`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// Module of the imported name.
    module: TypeModule,
    /// The name that was imported.
    name: ItemStr,
    /// Alias of the name imported.
    alias: Option<ItemStr>,
}

impl Import {
    /// Configure the importe name with the specified alias.
    ///
    /// This implised that the import is not qualified.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let toks = quote! {
    ///     $(python::import("collections", "namedtuple").with_alias("nt"))
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "from collections import namedtuple as nt",
    ///         "",
    ///         "nt",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_alias<T>(self, alias: T) -> Self
    where
        T: Into<ItemStr>,
    {
        Self {
            alias: Some(alias.into()),
            ..self
        }
    }

    /// Indicate that the import is qualified (module prefixed).
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let toks = quote! {
    ///     $(python::import("collections", "namedtuple").qualified())
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import collections",
    ///         "",
    ///         "collections.namedtuple",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn qualified(self) -> Self {
        Self {
            module: self.module.qualified(),
            ..self
        }
    }

    /// Configure the imported name with the specified alias.
    ///
    /// This implies that the import is qualified.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let toks = quote! {
    ///     $(python::import("collections", "namedtuple").with_module_alias("c"))
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import collections as c",
    ///         "",
    ///         "c.namedtuple",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_module_alias<T>(self, module_alias: T) -> Self
    where
        T: Into<ItemStr>,
    {
        Self {
            module: self.module.with_alias(module_alias),
            ..self
        }
    }
}

/// The import of a Python module `import module`.
///
/// Created through the [import_module()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ImportModule {
    /// Module of the imported name.
    module: ItemStr,

    /// Alias of module imported.
    alias: Option<ItemStr>,
}

impl ImportModule {
    /// Set alias for imported module.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let toks = quote! {
    ///     $(python::import_module("collections").with_alias("c"))
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "import collections as c",
    ///         "",
    ///         "c",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_alias<N>(self, new_alias: N) -> Self
    where
        N: Into<ItemStr>,
    {
        Self {
            alias: Some(new_alias.into()),
            ..self
        }
    }
}

impl Python {
    fn imports(out: &mut Tokens, tokens: &Tokens) {
        let mut imported_from = BTreeMap::new();
        let mut imports = BTreeSet::new();

        for import in tokens.walk_imports() {
            match import {
                Any::Import(Import {
                    module,
                    alias,
                    name,
                }) => match module {
                    TypeModule::Qualified { module, alias } => {
                        imports.insert((module, alias));
                    }
                    TypeModule::Unqualified { module } => {
                        imported_from
                            .entry(module)
                            .or_insert_with(BTreeSet::new)
                            .insert((name, alias));
                    }
                },
                Any::ImportModule(ImportModule { module, alias }) => {
                    imports.insert((module, alias));
                }
            }
        }

        if imported_from.is_empty() && imports.is_empty() {
            return;
        }

        for (module, imports) in imported_from {
            out.push();

            let imports = imports
                .into_iter()
                .map(|(name, alias)| quote!($name$(if let Some(a) = alias => $[' ']as $a)))
                .collect::<Vec<_>>();

            if imports.len() == 1 {
                quote_in! {*out =>
                    from $module import $(imports.into_iter().next())
                }
            } else {
                quote_in! {*out =>
                    from $module import $(for i in imports join (, ) => $i)
                }
            }
        }

        for (module, alias) in imports {
            out.push();

            quote_in! {*out =>
                import $module$(if let Some(a) = alias => $[' ']as $a)
            }
        }

        out.line();
    }
}

/// The import of a Python name `from module import foo`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let toks = quote! {
///     $(python::import("collections", "namedtuple").with_alias("nt"))
///     $(python::import("collections", "namedtuple"))
///     $(python::import("collections", "namedtuple").qualified())
///     $(python::import("collections", "namedtuple").with_module_alias("c"))
/// };
///
/// assert_eq!(
///     vec![
///         "from collections import namedtuple, namedtuple as nt",
///         "import collections",
///         "import collections as c",
///         "",
///         "nt",
///         "namedtuple",
///         "collections.namedtuple",
///         "c.namedtuple",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn import<M, N>(module: M, name: N) -> Import
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import {
        module: TypeModule::Unqualified {
            module: module.into(),
        },
        name: name.into(),
        alias: None,
    }
}

/// The import of a Python module `import module`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let toks = quote! {
///     $(python::import_module("collections"))
///     $(python::import_module("collections").with_alias("c"))
/// };
///
/// assert_eq!(
///     vec![
///         "import collections",
///         "import collections as c",
///         "",
///         "collections",
///         "c",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn import_module<M>(module: M) -> ImportModule
where
    M: Into<ItemStr>,
{
    ImportModule {
        module: module.into(),
        alias: None,
    }
}
