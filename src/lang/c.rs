//! Specialization for C code generation.

use core::fmt::Write as _;

use alloc::collections::BTreeSet;

use crate as genco;
use crate::fmt;
use crate::quote_in;
use crate::tokens::{quoted, ItemStr};

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

/// The include statement for a C header file such as `#include "foo/bar.h"` or
/// `#include <stdio.h>`.
///
/// Created using the [include()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// Path to included file.
    path: ItemStr,
    /// Item declared in the included file.
    item: ItemStr,
    /// True if the include is specified as a system header using `<>`, false if a local header using `""`.
    system: bool,
}

/// Format for C.
#[derive(Debug, Default)]
pub struct Format {}

/// Config data for C.
#[derive(Debug, Default)]
pub struct Config {}

impl C {
    fn imports(out: &mut Tokens, tokens: &Tokens) {
        let mut includes = BTreeSet::new();

        for include in tokens.walk_imports() {
            includes.insert((&include.path, include.system));
        }

        if includes.is_empty() {
            return;
        }

        for (file, system_header) in includes {
            if system_header {
                quote_in!(*out => #include <$(file)>);
            } else {
                quote_in!(*out => #include $(quoted(file)));
            }
            out.push();
        }

        out.line();
    }
}

/// Include an item declared in a local C header file such as `#include "foo/bar.h"`
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let fizzbuzz = c::include("foo/bar.h", "fizzbuzz");
///
/// let fizzbuzz_toks = quote! {
///     $fizzbuzz
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
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn include<M, N>(path: M, item: N) -> Import
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import {
        path: path.into(),
        item: item.into(),
        system: false,
    }
}

/// Include an item declared in a C system header such as `#include <stdio.h>`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let printf = c::include_system("stdio.h", "printf");
///
/// let printf_toks = quote! {
///     $printf
/// };
///
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
pub fn include_system<M, N>(path: M, item: N) -> Import
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import {
        path: path.into(),
        item: item.into(),
        system: true,
    }
}
