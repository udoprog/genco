//! Specialization for Java code generation.
//!
//! # String Quoting in Java
//!
//! Since Java uses UTF-16 internally, string quoting for high unicode
//! characters is done through surrogate pairs, as seen with the 😊 below.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: java::Tokens = quote!("start π 😊 \n \x7f end");
//! assert_eq!("\"start \\u03c0 \\ud83d\\ude0a \\n \\u007f end\"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```

mod block_comment;
pub use self::block_comment::BlockComment;

use core::fmt::Write as _;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};

use crate as genco;
use crate::fmt;
use crate::tokens::ItemStr;
use crate::{quote, quote_in};

/// Tokens container specialized for Java.
pub type Tokens = crate::Tokens<Java>;

impl_lang! {
    /// Language specialization for Java.
    pub Java {
        type Config = Config;
        type Format = Format;
        type Item = Import;

        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
            // From: https://docs.oracle.com/javase/tutorial/java/data/characters.html

            for c in input.chars() {
                match c {
                    '\t' => out.write_str("\\t")?,
                    '\u{0008}' => out.write_str("\\b")?,
                    '\n' => out.write_str("\\n")?,
                    '\r' => out.write_str("\\r")?,
                    '\u{0014}' => out.write_str("\\f")?,
                    '\'' => out.write_str("\\'")?,
                    '"' => out.write_str("\\\"")?,
                    '\\' => out.write_str("\\\\")?,
                    ' ' => out.write_char(' ')?,
                    c if c.is_ascii() && !c.is_control() => out.write_char(c)?,
                    c => {
                        for c in c.encode_utf16(&mut [0u16; 2]) {
                            write!(out, "\\u{c:04x}")?;
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

            if let Some(ref package) = config.package {
                quote_in!(header => package $package;);
                header.line();
            }

            let mut format = Format::default();
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
            let pkg = Some(self.package.as_ref());

            if &*self.package != JAVA_LANG && imported != pkg && file_package != pkg {
                out.write_str(self.package.as_ref())?;
                out.write_str(SEP)?;
            }

            out.write_str(&self.name)?;
            Ok(())
        }
    }
}

const JAVA_LANG: &str = "java.lang";
const SEP: &str = ".";

/// Formtat state for Java.
#[derive(Debug, Default)]
pub struct Format {
    /// Types which has been imported into the local namespace.
    imported: BTreeMap<String, String>,
}

/// Configuration for Java.
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
    /// let optional = java::import("java.util", "Optional");
    ///
    /// let toks = quote!($optional);
    ///
    /// let config = java::Config::default().with_package("java.util");
    /// let fmt = fmt::Config::from_lang::<Java>();
    ///
    /// let mut w = fmt::VecWriter::new();
    ///
    /// toks.format_file(&mut w.as_formatter(&fmt), &config)?;
    ///
    /// assert_eq!(
    ///     vec![
    ///         "package java.util;",
    ///         "",
    ///         "Optional",
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

/// The import of a Java type `import java.util.Optional;`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// Package of the class.
    package: ItemStr,
    /// Name  of class.
    name: ItemStr,
}

impl Java {
    fn imports(
        out: &mut Tokens,
        tokens: &Tokens,
        config: &Config,
        imported: &mut BTreeMap<String, String>,
    ) {
        let mut modules = BTreeSet::new();

        let file_package = config.package.as_ref().map(|p| p.as_ref());

        for import in tokens.walk_imports() {
            modules.insert((import.package.clone(), import.name.clone()));
        }

        if modules.is_empty() {
            return;
        }

        for (package, name) in modules {
            if imported.contains_key(&*name) {
                continue;
            }

            if &*package == JAVA_LANG {
                continue;
            }

            if Some(&*package) == file_package {
                continue;
            }

            out.append(quote!(import $(package.clone())$(SEP)$(name.clone());));
            out.push();

            imported.insert(name.to_string(), package.to_string());
        }

        out.line();
    }
}

/// The import of a Java type `import java.util.Optional;`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let integer = java::import("java.lang", "Integer");
/// let a = java::import("java.io", "A");
///
/// let toks = quote! {
///     $integer
///     $a
/// };
///
/// assert_eq!(
///     vec![
///         "import java.io.A;",
///         "",
///         "Integer",
///         "A",
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

/// Format a block comment, starting with `/**`, and ending in `*/`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
/// use std::iter;
///
/// let toks = quote! {
///     $(java::block_comment(vec!["first line", "second line"]))
///     $(java::block_comment(iter::empty::<&str>()))
///     $(java::block_comment(vec!["third line"]))
/// };
///
/// assert_eq!(
///     vec![
///         "/**",
///         " * first line",
///         " * second line",
///         " */",
///         "/**",
///         " * third line",
///         " */",
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
