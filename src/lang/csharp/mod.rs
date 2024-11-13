//! Specialization for Csharp code generation.
//!
//! # String Quoting in C#
//!
//! Since C# uses UTF-16 internally, but literal strings support C-style family
//! of escapes.
//!
//! See [c_family_write_quoted][super::c_family_write_quoted].
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

use core::fmt::Write as _;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};

use crate as genco;
use crate::fmt;
use crate::quote_in;
use crate::tokens::ItemStr;

pub use self::block_comment::BlockComment;
pub use self::comment::Comment;

/// Tokens container specialization for C#.
pub type Tokens = crate::Tokens<Csharp>;

impl_lang! {
    /// Language specialization for C#.
    pub Csharp {
        type Config = Config;
        type Format = Format;
        type Item = Import;

        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
            // From: https://csharpindepth.com/articles/Strings
            super::c_family_write_quoted(out, input)
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
                    namespace $namespace {
                        $tokens
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

    Import {
        fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
            {
                let qualified = self.qualified || is_qualified(config, format, &self.namespace, &self.name);

                if qualified {
                    out.write_str(&self.namespace)?;
                    out.write_str(SEP)?;
                }
            }

            out.write_str(&self.name)?;

            return Ok(());

            fn is_qualified(config: &Config, format: &Format, namespace: &str, name: &str) -> bool {
                // Name is in current namespace. No need to qualify.
                if let Some(config) = &config.namespace {
                    if &**config == namespace {
                        return false;
                    }
                }

                if let Some(imported) = format.imported_names.get(name) {
                    // a conflicting name is in the namespace.
                    if imported != namespace {
                        return true;
                    }
                }

                false
            }
        }
    }
}

/// Separator between types and modules in C#.
const SEP: &str = ".";

/// State using during formatting of C# language items.
#[derive(Debug, Default)]
pub struct Format {
    /// Keeping track of names which have been imported, do determine whether
    /// their use has to be qualified or not.
    ///
    /// A missing name means that it has to be used in a qualified manner.
    imported_names: BTreeMap<String, String>,
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
        }
    }
}

/// The import of a C# type `using System.IO;`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// namespace of the class.
    namespace: ItemStr,
    /// Name  of class.
    name: ItemStr,
    /// Use as qualified type.
    qualified: bool,
}

impl Import {
    /// Make this type into a qualified type that is always used with a
    /// namespace.
    pub fn qualified(self) -> Self {
        Self {
            qualified: true,
            ..self
        }
    }
}

impl Csharp {
    fn imports(
        out: &mut Tokens,
        tokens: &Tokens,
        config: &Config,
        imported_names: &mut BTreeMap<String, String>,
    ) {
        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            modules.insert((&*import.namespace, &*import.name));
        }

        if modules.is_empty() {
            return;
        }

        let mut imported = BTreeSet::new();

        for (namespace, name) in modules {
            if Some(namespace) == config.namespace.as_deref() {
                continue;
            }

            match imported_names.get(name) {
                // already imported...
                Some(existing) if existing == namespace => continue,
                // already imported, as something else...
                Some(_) => continue,
                _ => {}
            }

            if !imported.contains(namespace) {
                quote_in!(*out => using $namespace;);
                out.push();
                imported.insert(namespace);
            }

            imported_names.insert(name.to_string(), namespace.to_string());
        }

        out.line();
    }
}

/// The import of a C# type `using System.IO;`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let a = csharp::import("Foo.Bar", "A");
/// let b = csharp::import("Foo.Bar", "B");
/// let ob = csharp::import("Foo.Baz", "B");
///
/// let toks: Tokens<Csharp> = quote! {
///     $a
///     $b
///     $ob
/// };
///
/// assert_eq!(
///     vec![
///         "using Foo.Bar;",
///         "",
///         "A",
///         "B",
///         "Foo.Baz.B",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn import<P, N>(namespace: P, name: N) -> Import
where
    P: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import {
        namespace: namespace.into(),
        name: name.into(),
        qualified: false,
    }
}

/// Format a doc comment where each line is preceeded by `///`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
/// use std::iter;
///
/// let toks = quote! {
///     $(csharp::block_comment(vec!["Foo"]))
///     $(csharp::block_comment(iter::empty::<&str>()))
///     $(csharp::block_comment(vec!["Bar"]))
/// };
///
/// assert_eq!(
///     vec![
///         "/// Foo",
///         "/// Bar",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn block_comment<T>(comment: T) -> BlockComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    BlockComment(comment)
}

/// Format a doc comment where each line is preceeded by `//`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let toks = quote! {
///     $(csharp::comment(&["Foo"]))
///     $(csharp::comment(&["Bar"]))
/// };
///
/// assert_eq!(
///     vec![
///         "// Foo",
///         "// Bar",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn comment<T>(comment: T) -> Comment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    Comment(comment)
}
