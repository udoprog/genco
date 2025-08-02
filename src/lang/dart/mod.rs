//! Specialization for Dart code generation.
//!
//! # String Quoting in Dart
//!
//! Since Java uses UTF-16 internally, string quoting for high unicode
//! characters is done through surrogate pairs, as seen with the 😊 below.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: dart::Tokens = quote!("start π 😊 \n \x7f ÿ $ \\ end");
//! assert_eq!("\"start π 😊 \\n \\x7f ÿ \\$ \\\\ end\"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```
//!
//! # String Interpolation in Dart
//!
//! Strings can be interpolated in Dart, by using the special `$_(<string>)`
//! escape sequence.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: dart::Tokens = quote!($[str](  Hello: $var  ));
//! assert_eq!("\"  Hello: $var  \"", toks.to_string()?);
//!
//! let toks: dart::Tokens = quote!($[str](  Hello: $(a + b)  ));
//! assert_eq!("\"  Hello: ${a + b}  \"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```

mod doc_comment;
pub use self::doc_comment::DocComment;

use core::fmt::Write as _;

use alloc::collections::BTreeSet;

use crate as genco;
use crate::fmt;
use crate::quote_in;
use crate::tokens::{quoted, ItemStr};

const SEP: &str = ".";
/// dart:core package.
const DART_CORE: &str = "dart:core";

/// Tokens container specialization for Dart.
pub type Tokens = crate::Tokens<Dart>;

impl genco::lang::LangSupportsEval for Dart {}

impl_lang! {
    /// Language specialization for Dart.
    pub Dart {
        type Config = Config;
        type Format = Format;
        type Item = Import;

        fn string_eval_literal(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
            literal: &str,
        ) -> fmt::Result {
            write!(out, "${literal}")?;
            Ok(())
        }

        /// Start a string-interpolated eval.
        fn start_string_eval(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
        ) -> fmt::Result {
            out.write_str("${")?;
            Ok(())
        }

        /// End a string interpolated eval.
        fn end_string_eval(
            out: &mut fmt::Formatter<'_>,
            _config: &Self::Config,
            _format: &Self::Format,
        ) -> fmt::Result {
            out.write_char('}')?;
            Ok(())
        }

        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
            // Note: Dart is like C escape, but since it supports string
            // interpolation, `$` also needs to be escaped!

            for c in input.chars() {
                match c {
                    // backspace
                    '\u{0008}' => out.write_str("\\b")?,
                    // form feed
                    '\u{0012}' => out.write_str("\\f")?,
                    // new line
                    '\n' => out.write_str("\\n")?,
                    // carriage return
                    '\r' => out.write_str("\\r")?,
                    // horizontal tab
                    '\t' => out.write_str("\\t")?,
                    // vertical tab
                    '\u{0011}' => out.write_str("\\v")?,
                    // Note: only relevant if we were to use single-quoted strings.
                    // '\'' => out.write_str("\\'")?,
                    '"' => out.write_str("\\\"")?,
                    '\\' => out.write_str("\\\\")?,
                    '$' => out.write_str("\\$")?,
                    c if !c.is_control() => out.write_char(c)?,
                    c if (c as u32) < 0x100 => {
                        write!(out, "\\x{:02x}", c as u32)?;
                    }
                    c => {
                        for c in c.encode_utf16(&mut [0u16; 2]) {
                            write!(out, "\\u{c:04x}")?;
                        }
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
            let mut imports: Tokens = Tokens::new();
            Self::imports(&mut imports, tokens, config);
            let format = Format::default();
            imports.format(out, config, &format)?;
            tokens.format(out, config, &format)?;
            Ok(())
        }
    }

    Import {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            if let Some(alias) = &self.alias {
                out.write_str(alias.as_ref())?;
                out.write_str(SEP)?;
            }

            out.write_str(&self.name)?;
            Ok(())
        }
    }
}

/// Format state for Dart.
#[derive(Debug, Default)]
pub struct Format {}

/// Config data for Dart formatting.
#[derive(Debug, Default)]
pub struct Config {}

/// The import of a Dart type `import "dart:math";`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Import {
    /// Path to import.
    path: ItemStr,
    /// Name imported.
    name: ItemStr,
    /// Alias of module.
    alias: Option<ItemStr>,
}

impl Import {
    /// Add an `as` keyword to the import.
    pub fn with_alias(self, alias: impl Into<ItemStr>) -> Import {
        Self {
            alias: Some(alias.into()),
            ..self
        }
    }
}

impl Dart {
    /// Resolve all imports.
    fn imports(out: &mut Tokens, input: &Tokens, _: &Config) {
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
                quote_in!(*out => import $(quoted(name)) as $alias;);
            } else {
                quote_in!(*out => import $(quoted(name)););
            }

            out.push();
        }

        out.line();
    }
}

/// The import of a Dart type `import "dart:math";`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let a = dart::import("package:http/http.dart", "A");
/// let b = dart::import("package:http/http.dart", "B");
/// let c = dart::import("package:http/http.dart", "C").with_alias("h2");
/// let d = dart::import("../http.dart", "D");
///
/// let toks = quote! {
///     $a
///     $b
///     $c
///     $d
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
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn import<P, N>(path: P, name: N) -> Import
where
    P: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import {
        path: path.into(),
        alias: None,
        name: name.into(),
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
///     $(dart::doc_comment(vec!["Foo"]))
///     $(dart::doc_comment(iter::empty::<&str>()))
///     $(dart::doc_comment(vec!["Bar"]))
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
pub fn doc_comment<T>(comment: T) -> DocComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    DocComment(comment)
}
