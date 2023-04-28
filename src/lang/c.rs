//! Specialization for C code generation.

use crate as genco;
use crate::fmt;
use crate::quote_in;
use crate::tokens::{quoted, ItemStr};
use std::collections::BTreeSet;
use std::fmt::Write as _;

/// Tokens container specialization for C.
pub type Tokens = crate::Tokens<C>;

impl_lang! {
    /// Language specialization for C.
    pub C {
        type Config = Config;
        type Format = Format;
        type Item = Import;

        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
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
            out.write_str(&self.item)?;
            Ok(())
        }
    }
}

/// The include of a C file `#include "foo/bar.h"`.
///
/// Created using the [include()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// Path to included file.
    path: ItemStr,
    /// Item declared in the included file.
    item: ItemStr,
    /// Whether the import is specified as a relative path `""` or as a system include `<>`
    relative: bool,
}

/// Format for C.
#[derive(Debug, Default)]
pub struct Format {}

/// Config data for C.
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

impl C {
    fn imports(out: &mut Tokens, tokens: &Tokens) {
        let mut includes = BTreeSet::new();

        for include in tokens.walk_imports() {
            includes.insert((&include.path, include.relative));
        }

        if includes.is_empty() {
            return;
        }

        for (file, relative) in includes {
            if relative {
                quote_in!(*out => #include $(quoted(file)));
            } else {
                quote_in!(*out => #include <$(file)>);
            }
            out.push();
        }

        out.line();
    }
}

/// Including a C header file `#include "foo/bar.h"`, and standard library
/// header `#include <stdio.h>`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let fizzbuzz = c::include("foo/bar.h", "fizzbuzz", true);
/// let printf = c::include("stdio.h", "printf", false);
///
/// let fizzbuzz_toks = quote! {
///     $fizzbuzz
/// };
/// let printf_toks = quote! {
///     $printf
/// };
///
/// assert_eq!(
///     vec![
///        "#include \"foo/bar.h\"",
///        "",
///        "fizzbuzz",
///     ],
///     fizzbuzz_toks.to_file_vec()?
/// );
/// assert_eq!(
///     vec![
///        "#include <stdio.h>",
///        "",
///        "printf",
///     ],
///     printf_toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn include<M, N>(path: M, item: N, relative: bool) -> Import
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import {
        path: path.into(),
        item: item.into(),
        relative,
    }
}
