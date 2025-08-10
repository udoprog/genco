//! Specialization for Kotlin code generation.
//!
//! # String Quoting in Kotlin
//!
//! Since Kotlin runs on the JVM, it also uses UTF-16 internally. String
//! quoting for high unicode characters is done through surrogate pairs, as
//! seen with the 😊 emoji below. Kotlin also requires escaping `$` characters
//! in standard string literals.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: kotlin::Tokens = quote!("start π 😊 $var \n end");
//! assert_eq!("\"start \\u03c0 \\ud83d\\ude0a \\$var \\n end\"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```

use core::fmt::Write as _;

use crate as genco;
use crate::fmt;
use crate::quote_in;
use crate::tokens::ItemStr;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};

/// Tokens container specialized for Kotlin.
pub type Tokens = crate::Tokens<Kotlin>;

// This trait implementation signals to genco that the Kotlin language
// supports evaluation constructs like `$(if ...)` in `quote!`.
impl genco::lang::LangSupportsEval for Kotlin {}

impl_lang! {
    /// Language specialization for Kotlin.
    pub Kotlin {
        type Config = Config;
        type Format = Format;
        type Item = Import;

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
            // See: https://kotlinlang.org/docs/basic-types.html#escaped-strings
            for c in input.chars() {
                match c {
                    '\t' => out.write_str("\\t")?,
                    '\u{0008}' => out.write_str("\\b")?, // Backspace
                    '\n' => out.write_str("\\n")?,
                    '\r' => out.write_str("\\r")?,
                    '\'' => out.write_str("\\'")?,
                    '"' => out.write_str("\\\"")?,
                    '\\' => out.write_str("\\\\")?,
                    '$' => out.write_str("\\$")?,
                    c if c.is_ascii() && !c.is_control() => out.write_char(c)?,
                    c => {
                        // Encode non-ascii characters as UTF-16 surrogate pairs
                        for unit in c.encode_utf16(&mut [0u16; 2]) {
                            write!(out, "\\u{unit:04x}")?;
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
            let mut format = Format::default();

            if let Some(ref package) = config.package {
                // package declarations in Kotlin do not have semicolons
                quote_in!(header => package $package);
                header.line();
            }

            Self::imports(&mut header, tokens, config, &mut format.imported);
            header.format(out, config, &format)?;
            tokens.format(out, config, &format)?;
            Ok(())
        }
    }

    Import {
        fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, format: &Format) -> fmt::Result {
            let file_package = config.package.as_ref().map(|p| p.as_ref());
            let imported = format.imported.get(self.name.as_ref()).map(String::as_str);
            let current_package = Some(self.package.as_ref());

            // Determine if we need to use the fully qualified name (FQN).
            // Use FQN if the class is not in the current package and has not been imported.
            // Or if a class with the same name has been imported from a different package.
            if file_package != current_package && imported != current_package {
                out.write_str(self.package.as_ref())?;
                out.write_str(".")?;
            }

            out.write_str(&self.name)?;
            Ok(())
        }
    }
}

/// Formatting state for Kotlin.
#[derive(Debug, Default)]
pub struct Format {
    /// Types which have been imported into the local namespace.
    /// Maps a simple name to its full package.
    imported: BTreeMap<String, String>,
}

/// Configuration for Kotlin.
#[derive(Debug, Default)]
pub struct Config {
    /// Package to use.
    package: Option<ItemStr>,
}

impl Config {
    /// Configure package to use for the file generated.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    /// use genco::fmt;
    ///
    /// let list = kotlin::import("kotlin.collections", "List");
    ///
    /// let toks = quote!($list);
    ///
    /// let config = kotlin::Config::default().with_package("com.example");
    /// let fmt = fmt::Config::from_lang::<Kotlin>();
    ///
    /// let mut w = fmt::VecWriter::new();
    ///
    /// toks.format_file(&mut w.as_formatter(&fmt), &config)?;
    ///
    /// assert_eq!(
    ///     vec![
    ///         "package com.example",
    ///         "",
    ///         "import kotlin.collections.List",
    ///         "",
    ///         "List",
    ///     ],
    ///     w.into_vec(),
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_package<P>(self, package: P) -> Self
    where
        P: Into<ItemStr>,
    {
        Self {
            package: Some(package.into()),
        }
    }
}

/// An import of a Kotlin type, like `import kotlin.collections.List`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// Package of the class.
    package: ItemStr,
    /// Name of the class.
    name: ItemStr,
}

impl Kotlin {
    /// Gathers and writes import statements to the header.
    fn imports(
        out: &mut Tokens,
        tokens: &Tokens,
        config: &Config,
        imported: &mut BTreeMap<String, String>,
    ) {
        let mut to_import = BTreeSet::new();
        let file_package = config.package.as_ref().map(|p| p.as_ref());

        for import in tokens.walk_imports() {
            // Don't import if the type is in the current package
            if Some(import.package.as_ref()) == file_package {
                continue;
            }

            // Don't import if a class with the same name is already imported from another package.
            if let Some(existing_package) = imported.get(import.name.as_ref()) {
                if existing_package != import.package.as_ref() {
                    continue;
                }
            }

            to_import.insert(import.clone());
        }

        if to_import.is_empty() {
            return;
        }

        for import in to_import {
            // import statements in Kotlin do not have semicolons
            quote_in!(*out => import $(&import.package).$(&import.name));
            out.push();
            imported.insert(import.name.to_string(), import.package.to_string());
        }

        out.line();
    }
}

/// Create a new Kotlin import.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let list = kotlin::import("kotlin.collections", "List");
/// let map = kotlin::import("kotlin.collections", "Map");
///
/// let toks = quote! {
///     val a: $list<String>
///     val b: $map<String, Int>
/// };
///
/// assert_eq!(
///     vec![
///         "import kotlin.collections.List",
///         "import kotlin.collections.Map",
///         "",
///         "val a: List<String>",
///         "val b: Map<String, Int>",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn import<P, N>(package: P, name: N) -> Import
where
    P: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import {
        package: package.into(),
        name: name.into(),
    }
}
