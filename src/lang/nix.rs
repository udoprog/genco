//! Nix

use core::fmt::Write as _;

use alloc::collections::BTreeSet;
use alloc::string::ToString;

use crate as genco;
use crate::fmt;
use crate::quote_in;
use crate::tokens::ItemStr;

/// Tokens
pub type Tokens = crate::Tokens<Nix>;

impl_lang! {
    /// Nix
    pub Nix {
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

            if !config.scoped {
                Self::arguments(&mut header, tokens);
            }
            Self::withs(&mut header, tokens);
            Self::imports(&mut header, tokens);
            let format = Format::default();
            header.format(out, config, &format)?;
            tokens.format(out, config, &format)?;
            Ok(())
        }
    }

    Import {
        fn format(&self, out: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            match self {
                Import::Argument(import) => out.write_str(&import.0)?,
                Import::Inherit(import) => out.write_str(&import.name)?,
                Import::Variable(import) => out.write_str(&import.name)?,
                Import::With(import) => out.write_str(&import.name)?,
            }
            Ok(())
        }
    }
}

/// Import
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Import {
    /// Argument
    Argument(ImportArgument),
    /// Inherit
    Inherit(ImportInherit),
    /// Variable
    Variable(ImportVariable),
    /// With
    With(ImportWith),
}

/// ImportArgument
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ImportArgument(ItemStr);

/// ImportInherit
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ImportInherit {
    /// Path
    path: ItemStr,
    /// Name
    name: ItemStr,
}

/// ImportVariable
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ImportVariable {
    /// Name
    name: ItemStr,
    /// Value
    value: Tokens,
}

/// ImportWith
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ImportWith {
    /// Argument
    argument: ItemStr,
    /// Name
    name: ItemStr,
}

/// Format
#[derive(Debug, Default)]
pub struct Format {}

/// Nix formatting configuration.
#[derive(Debug, Default)]
pub struct Config {
    scoped: bool,
}

impl Config {
    /// With scoped
    pub fn with_scoped(self, scoped: bool) -> Self {
        Self { scoped }
    }
}

impl Nix {
    fn arguments(out: &mut Tokens, tokens: &Tokens) {
        let mut arguments = BTreeSet::new();

        for imports in tokens.walk_imports() {
            match imports {
                Import::Argument(argument) => {
                    arguments.insert(argument.0.to_string());
                }
                Import::Inherit(inherit) => {
                    let argument = inherit.path.split('.').next();
                    if let Some(a) = argument {
                        arguments.insert(a.to_string());
                    }
                }
                Import::Variable(variable) => {
                    let value = &variable.value;
                    for import in value.walk_imports() {
                        match import {
                            Import::Inherit(inherit) => {
                                let argument = inherit.path.split('.').next();
                                if let Some(a) = argument {
                                    arguments.insert(a.to_string());
                                }
                            }
                            Import::Argument(argument) => {
                                arguments.insert(argument.0.to_string());
                            }
                            _ => (),
                        }
                    }
                }
                Import::With(with) => {
                    let argument = with.argument.split('.').next();
                    if let Some(a) = argument {
                        arguments.insert(a.to_string());
                    }
                }
            }
        }

        out.append("{");
        out.push();
        out.indent();

        for argument in arguments {
            quote_in!(*out => $argument,);
            out.push();
        }

        out.append("...");
        out.push();

        out.unindent();
        out.append("}:");
        out.push();

        out.line();
    }

    fn withs(out: &mut Tokens, tokens: &Tokens) {
        let mut withs = BTreeSet::new();

        for imports in tokens.walk_imports() {
            if let Import::With(with) = imports {
                withs.insert(&with.argument);
            }
        }

        if withs.is_empty() {
            return;
        }

        for name in withs {
            quote_in!(*out => with $name;);
            out.push();
        }

        out.line();
    }

    fn imports(out: &mut Tokens, tokens: &Tokens) {
        let mut inherits = BTreeSet::new();
        let mut variables = BTreeSet::new();

        for imports in tokens.walk_imports() {
            match imports {
                Import::Inherit(inherit) => {
                    inherits.insert((&inherit.path, &inherit.name));
                }
                Import::Variable(variable) => {
                    let value = &variable.value;
                    for import in value.walk_imports() {
                        if let Import::Inherit(inherit) = import {
                            inherits.insert((&inherit.path, &inherit.name));
                        }
                    }
                    variables.insert((&variable.name, &variable.value));
                }
                _ => (),
            }
        }

        if inherits.is_empty() && variables.is_empty() {
            return;
        }

        out.append("let");
        out.push();
        out.indent();

        for (path, name) in inherits {
            quote_in!(*out => inherit ($path) $name;);
            out.push();
        }

        for (name, value) in variables {
            quote_in!(*out => $name = $value;);
            out.push();
        }

        out.unindent();
        out.append("in");
        out.push();

        out.line();
    }
}

/// ```
/// use genco::prelude::*;
///
/// let cell = nix::argument("cell");
///
/// let toks = quote! {
///     $cell
/// };
///
/// assert_eq!(
///     vec![
///         "{",
///         "    cell,",
///         "    ...",
///         "}:",
///         "",
///         "cell",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn argument<M>(name: M) -> Import
where
    M: Into<ItemStr>,
{
    Import::Argument(ImportArgument(name.into()))
}

/// ```
/// use genco::prelude::*;
///
/// let nixpkgs = nix::inherit("inputs", "nixpkgs");
///
/// let toks = quote! {
///     $nixpkgs
/// };
///
/// assert_eq!(
///     vec![
///         "{",
///         "    inputs,",
///         "    ...",
///         "}:",
///         "",
///         "let",
///         "    inherit (inputs) nixpkgs;",
///         "in",
///         "",
///         "nixpkgs",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn inherit<M, N>(path: M, name: N) -> Import
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import::Inherit(ImportInherit {
        path: path.into(),
        name: name.into(),
    })
}

/// ```
/// use genco::prelude::*;
///
/// let nixpkgs = &nix::inherit("inputs", "nixpkgs");
///
/// let pkgs = nix::variable("pkgs", quote! {
///     import $nixpkgs {
///         inherit ($nixpkgs) system;
///         config.allowUnfree = true;
///     }
/// });
///
/// let toks = quote! {
///     $pkgs
/// };
///
/// assert_eq!(
///     vec![
///         "{",
///         "    inputs,",
///         "    ...",
///         "}:",
///         "",
///         "let",
///         "    inherit (inputs) nixpkgs;",
///         "    pkgs = import nixpkgs {",
///         "        inherit (nixpkgs) system;",
///         "        config.allowUnfree = true;",
///         "    };",
///         "in",
///         "",
///         "pkgs"
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn variable<M, N>(name: M, value: N) -> Import
where
    M: Into<ItemStr>,
    N: Into<Tokens>,
{
    Import::Variable(ImportVariable {
        name: name.into(),
        value: value.into(),
    })
}

/// ```
/// use genco::prelude::*;
///
/// let concat_map = nix::with("inputs.nixpkgs.lib", "concatMap");
/// let list_to_attrs = nix::with("inputs.nixpkgs.lib", "listToAttrs");
///
/// let toks = quote! {
///     $list_to_attrs $concat_map
/// };
///
/// assert_eq!(
///     vec![
///         "{",
///         "    inputs,",
///         "    ...",
///         "}:",
///         "",
///         "with inputs.nixpkgs.lib;",
///         "",
///         "listToAttrs concatMap",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn with<M, N>(argument: M, name: N) -> Import
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Import::With(ImportWith {
        argument: argument.into(),
        name: name.into(),
    })
}
