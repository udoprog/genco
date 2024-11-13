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

use core::fmt::Write as _;

use alloc::collections::BTreeSet;

use crate::fmt;
use crate::tokens::ItemStr;

/// Tokens container specialization for Rust.
pub type Tokens = crate::Tokens<Swift>;

impl_lang! {
    /// Swift token specialization.
    pub Swift {
        type Config = Config;
        type Format = Format;
        type Item = Import;

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

    Import {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str(&self.name)
        }
    }
}

/// Format state for Swift code.
#[derive(Debug, Default)]
pub struct Format {}

/// Configuration for formatting Swift code.
#[derive(Debug, Default)]
pub struct Config {}

/// The import of a Swift type `import UIKit`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// Module of the imported name.
    module: ItemStr,
    /// Name imported.
    name: ItemStr,
}

impl Swift {
    fn imports(out: &mut Tokens, tokens: &Tokens) {
        use crate as genco;
        use crate::quote_in;

        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            modules.insert(&import.module);
        }

        if !modules.is_empty() {
            for module in modules {
                quote_in! { *out => $['\r']import $module}
            }
        }

        out.line();
    }
}

/// The import of a Swift type `import UIKit`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let toks = quote!($(swift::import("Foo", "Debug")));
///
/// assert_eq!(
///     vec![
///         "import Foo",
///         "",
///         "Debug",
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
        module: module.into(),
        name: name.into(),
    }
}
