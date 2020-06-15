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
use crate::lang::Lang;
use crate::quote_in;
use crate::tokens::ItemStr;
use std::collections::{BTreeSet, HashMap, HashSet};

pub use self::block_comment::BlockComment;
pub use self::comment::Comment;

/// Tokens container specialization for C#.
pub type Tokens = crate::Tokens<Csharp>;

impl_dynamic_types! {
    /// Language specialization for C#.
    pub Csharp
    =>
    trait TypeTrait {
    }

    Import {
        impl TypeTrait {
        }

        impl LangItem {
            fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
                {
                    let qualified = self.qualified || is_qualified(config, format, &*self.namespace, &*self.name);

                    if qualified {
                        out.write_str(&self.namespace)?;
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

            fn as_import(&self) -> Option<&Self> {
                Some(self)
            }
        }
    }
}

static SYSTEM: &'static str = "System";
static SEP: &'static str = ".";

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

/// A class.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// namespace of the class.
    namespace: ItemStr,
    /// Name  of class.
    name: ItemStr,
    /// Path of class when nested.
    path: Vec<ItemStr>,
    /// Use as qualified type.
    qualified: bool,
}

impl Import {
    /// Specify the path of the imported element.
    ///
    /// This discards any arguments associated with it.
    pub fn with_path(self, path: Vec<ItemStr>) -> Self {
        Self { path, ..self }
    }

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
        imported_names: &mut HashMap<String, String>,
    ) {
        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            if &*import.namespace != SYSTEM {
                modules.insert((&*import.namespace, &*import.name));
            }
        }

        if modules.is_empty() {
            return;
        }

        let mut imported = HashSet::new();

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
                quote_in!(*out => using #namespace;);
                out.push();
                imported.insert(namespace);
            }

            imported_names.insert(name.to_string(), namespace.to_string());
        }

        out.line();
    }
}

impl Lang for Csharp {
    type Config = Config;
    type Format = Format;
    type Import = Import;

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
/// let a = csharp::import("Foo.Bar", "A");
/// let b = csharp::import("Foo.Bar", "B");
/// let ob = csharp::import("Foo.Baz", "B");
///
/// let toks: Tokens<Csharp> = quote! {
///     #a
///     #b
///     #ob
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
/// # Ok(())
/// # }
/// ```
pub fn import<P, N>(namespace: P, name: N) -> Import
where
    P: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import {
        namespace: namespace.into(),
        name: name.into(),
        path: vec![],
        qualified: false,
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
