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
//! let toks: swift::Tokens = quote!("start π 😊 \n \x7f ÿ $ end");
//! assert_eq!("\"start π 😊 \\n \\u{7f} ÿ $ end\"", toks.to_string()?);
//! # Ok::<_, genco::fmt::Error>(())
//! ```

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
        type Item = Any;

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

    Import(Import) {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            out.write_str(&self.name)
        }
    }

    ImportImplementationOnly(ImportImplementationOnly) {
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

/// The implementation-only import of a Swift type `@_implementationOnly import UIKit`.
///
/// Created through the [import_implementation_only()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ImportImplementationOnly {
    /// Module of the imported name.
    module: ItemStr,
    /// Name imported.
    name: ItemStr,
}

/// The type of import statement to use when importing a Swift module.
/// - Standard imports that make the module's public API available
/// - Implementation-only imports that hide the imported module from clients
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
enum ImportType {
    /// A standard Swift import statement: `import ModuleName`
    Import,
    /// An implementation-only import statement: `@_implementationOnly import ModuleName`
    ///
    /// This type of import hides the imported module from the public API,
    /// preventing clients from depending on it transitively.
    ImportImplementationOnly,
}

impl Swift {
    fn imports(out: &mut Tokens, tokens: &Tokens) {
        use crate as genco;
        use crate::quote_in;

        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            match import.kind() {
                AnyKind::Import(ref i) => {
                    modules.insert((&i.module, ImportType::Import));
                }
                AnyKind::ImportImplementationOnly(ref i) => {
                    modules.insert((&i.module, ImportType::ImportImplementationOnly));
                }
            }
        }

        if !modules.is_empty() {
            for (module, import_type) in modules {
                match import_type {
                    ImportType::Import => {
                        quote_in! { *out => $['\r']import $module}
                    }
                    ImportType::ImportImplementationOnly => {
                        quote_in! { *out => $['\r']@_implementationOnly import $module}
                    }
                }
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

/// The implementation-only import of a Swift type `@_implementationOnly import UIKit`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let toks = quote!($(swift::import_implementation_only("Foo", "Debug")));
///
/// assert_eq!(
///     vec![
///         "@_implementationOnly import Foo",
///         "",
///         "Debug",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn import_implementation_only(
    module: impl Into<ItemStr>,
    name: impl Into<ItemStr>,
) -> ImportImplementationOnly {
    ImportImplementationOnly {
        module: module.into(),
        name: name.into(),
    }
}
