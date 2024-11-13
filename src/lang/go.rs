//! Specialization for Go code generation.
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
//! let toks: go::Tokens = quote!("start π 😊 \n \x7f end");
//! assert_eq!("\"start \\u03c0 \\U0001f60a \\n \\x7f end\"", toks.to_string()?);
//! # Ok(())
//! # }
//! ```

use core::fmt::Write as _;

use alloc::collections::BTreeSet;

use crate as genco;
use crate::fmt;
use crate::quote_in;
use crate::tokens::{quoted, ItemStr};

const MODULE_SEP: &str = "/";
const SEP: &str = ".";

/// Tokens container specialization for Go.
pub type Tokens = crate::Tokens<Go>;

impl_lang! {
    /// Language specialization for Go.
    pub Go {
        type Config = Config;
        type Format = Format;
        type Item = Import;

        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
            // From: https://golang.org/src/strconv/quote.go
            super::c_family_write_quoted(out, input)
        }

        fn format_file(
            tokens: &Tokens,
            out: &mut fmt::Formatter<'_>,
            config: &Self::Config,
        ) -> fmt::Result {
            let mut header = Tokens::new();

            if let Some(package) = &config.package {
                quote_in!(header => package $package);
                header.line();
            }

            Self::imports(&mut header, tokens);
            let format = Format::default();
            header.format(out, config, &format)?;
            tokens.format(out, config, &format)?;
            Ok(())
        }
    }

    Import {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            if let Some(module) = self.module.rsplit(MODULE_SEP).next() {
                out.write_str(module)?;
                out.write_str(SEP)?;
            }

            out.write_str(&self.name)?;
            Ok(())
        }
    }
}

/// The import of a Go type `import "foo/bar"`.
///
/// Created using the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// Module of the imported name.
    module: ItemStr,
    /// Name imported.
    name: ItemStr,
}

/// Format for Go.
#[derive(Debug, Default)]
pub struct Format {}

/// Config data for Go.
#[derive(Debug, Default)]
pub struct Config {
    package: Option<ItemStr>,
}

impl Config {
    /// Configure the specified package.
    pub fn with_package<P: Into<ItemStr>>(self, package: P) -> Self {
        Self {
            package: Some(package.into()),
        }
    }
}

impl Go {
    fn imports(out: &mut Tokens, tokens: &Tokens) {
        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            modules.insert(&import.module);
        }

        if modules.is_empty() {
            return;
        }

        for module in modules {
            quote_in!(*out => import $(quoted(module)));
            out.push();
        }

        out.line();
    }
}

/// The import of a Go type `import "foo/bar"`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let ty = go::import("foo/bar", "Debug");
///
/// let toks = quote! {
///     $ty
/// };
///
/// assert_eq!(
///     vec![
///        "import \"foo/bar\"",
///        "",
///        "bar.Debug",
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
