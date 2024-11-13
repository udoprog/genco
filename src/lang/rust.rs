//! Specialization for Rust code generation.
//!
//! # Examples
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: rust::Tokens = quote! {
//!     fn foo() -> u32 {
//!         42
//!     }
//! };
//!
//! assert_eq!(
//!     vec![
//!         "fn foo() -> u32 {",
//!         "    42",
//!         "}",
//!     ],
//!     toks.to_file_vec()?
//! );
//! # Ok(())
//! # }
//! ```
//!
//! # String Quoting in Rust
//!
//! Rust uses UTF-8 internally, string quoting is with the exception of escape
//! sequences a one-to-one translation.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let toks: rust::Tokens = quote!("start π 😊 \n \x7f ÿ $ end");
//! assert_eq!("\"start π 😊 \\n \\x7f ÿ $ end\"", toks.to_string()?);
//! # Ok(())
//! # }

use core::fmt::Write as _;

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::fmt;
use crate::tokens::ItemStr;

const SEP: &str = "::";

/// Tokens container specialization for Rust.
pub type Tokens = crate::Tokens<Rust>;

impl_lang! {
    /// Language specialization for Rust.
    pub Rust {
        type Config = Config;
        type Format = Format;
        type Item = Import;

        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
            // From: https://doc.rust-lang.org/reference/tokens.html#literals

            for c in input.chars() {
                match c {
                    // new line
                    '\n' => out.write_str("\\n")?,
                    // carriage return
                    '\r' => out.write_str("\\r")?,
                    // horizontal tab
                    '\t' => out.write_str("\\t")?,
                    // backslash
                    '\\' => out.write_str("\\\\")?,
                    // null
                    '\0' => out.write_str("\\0")?,
                    // Note: only relevant if we were to use single-quoted strings.
                    // '\'' => out.write_str("\\'")?,
                    '"' => out.write_str("\\\"")?,
                    c if !c.is_control() => out.write_char(c)?,
                    c if (c as u32) < 0x80 => {
                        write!(out, "\\x{:02x}", c as u32)?;
                    }
                    c => {
                        write!(out, "\\u{{{:04x}}}", c as u32)?;
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
            Self::imports(&mut imports, config, tokens);

            let format = Format::default();
            imports.format(out, config, &format)?;
            tokens.format(out, config, &format)?;
            Ok(())
        }
    }

    Import {
        fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, _: &Format) -> fmt::Result {
            match &self.module {
                Module::Module {
                    import: Some(ImportMode::Direct),
                    ..
                } => {
                    self.write_direct(out)?;
                }
                Module::Module {
                    import: Some(ImportMode::Qualified),
                    module,
                } => {
                    self.write_prefixed(out, module)?;
                }
                Module::Module {
                    import: None,
                    module,
                } => match &config.default_import {
                    ImportMode::Direct => self.write_direct(out)?,
                    ImportMode::Qualified => self.write_prefixed(out, module)?,
                },
                Module::Aliased {
                    alias: ref module, ..
                } => {
                    out.write_str(module)?;
                    out.write_str(SEP)?;
                    out.write_str(&self.name)?;
                }
            }

            Ok(())
        }
    }
}

/// Format state for Rust.
#[derive(Debug, Default)]
pub struct Format {}

/// Language configuration for Rust.
#[derive(Debug)]
pub struct Config {
    default_import: ImportMode,
}

impl Config {
    /// Configure the default import mode to use.
    ///
    /// See [Import] for more details.
    pub fn with_default_import(self, default_import: ImportMode) -> Self {
        Self { default_import }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_import: ImportMode::Direct,
        }
    }
}

/// The import mode to use when generating import statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportMode {
    /// Import names without a module prefix.
    ///
    /// so for `std::fmt::Debug` it would import `std::fmt::Debug`, and use
    /// `Debug`.
    Direct,
    /// Import qualified names with a module prefix.
    ///
    /// so for `std::fmt::Debug` it would import `std::fmt`, and use
    /// `fmt::Debug`.
    Qualified,
}

#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
enum Module {
    /// Type imported directly from module with the specified mode.
    Module {
        import: Option<ImportMode>,
        module: ItemStr,
    },
    /// Prefixed with an alias.
    Aliased { module: ItemStr, alias: ItemStr },
}

impl Module {
    /// Convert into an aliased import, or keep as same in case that's not
    /// feasible.
    fn into_module_aliased<A>(self, alias: A) -> Self
    where
        A: Into<ItemStr>,
    {
        match self {
            Self::Module { module, .. } => Self::Aliased {
                module,
                alias: alias.into(),
            },
            other => other,
        }
    }

    /// Aliasing a type explicitly means you no longer want to import it by
    /// module. Set the correct import here.
    fn into_aliased(self) -> Self {
        match self {
            Self::Module { module, .. } => Self::Module {
                import: Some(ImportMode::Direct),
                module,
            },
            other => other,
        }
    }

    /// Switch to a direct import mode.
    ///
    /// See [ImportMode::Direct].
    fn direct(self) -> Self {
        match self {
            Self::Module { module, .. } => Self::Module {
                module,
                import: Some(ImportMode::Direct),
            },
            other => other,
        }
    }

    /// Switch into a qualified import mode.
    ///
    /// See [ImportMode::Qualified].
    fn qualified(self) -> Self {
        match self {
            Self::Module { module, .. } => Self::Module {
                module,
                import: Some(ImportMode::Qualified),
            },
            other => other,
        }
    }
}

/// The import of a Rust type `use std::collections::HashMap`.
///
/// Created through the [import()] function.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Import {
    /// How the type is imported.
    module: Module,
    /// Name of type.
    name: ItemStr,
    /// Alias to use for the type.
    alias: Option<ItemStr>,
}

impl Import {
    /// Alias the given type as it's imported.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let ty = rust::import("std::fmt", "Debug").with_alias("FmtDebug");
    ///
    /// let toks = quote!($ty);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "use std::fmt::Debug as FmtDebug;",
    ///         "",
    ///         "FmtDebug",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn with_alias<A: Into<ItemStr>>(self, alias: A) -> Self {
        Self {
            module: self.module.into_aliased(),
            alias: Some(alias.into()),
            ..self
        }
    }

    /// Alias the module being imported.
    ///
    /// This also implies that the import is [qualified()].
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let ty = rust::import("std::fmt", "Debug").with_module_alias("other");
    ///
    /// let toks = quote!($ty);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "use std::fmt as other;",
    ///         "",
    ///         "other::Debug",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    ///
    /// [qualified()]: Self::qualified()
    pub fn with_module_alias<A: Into<ItemStr>>(self, alias: A) -> Self {
        Self {
            module: self.module.into_module_aliased(alias),
            ..self
        }
    }

    /// Switch to a qualified import mode.
    ///
    /// See [ImportMode::Qualified].
    ///
    /// So importing `std::fmt::Debug` will cause the module to be referenced as
    /// `fmt::Debug` instead of `Debug`.
    ///
    /// This is implied if [with_module_alias()][Self::with_module_alias()] is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let ty = rust::import("std::fmt", "Debug").qualified();
    ///
    /// let toks = quote!($ty);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "use std::fmt;",
    ///         "",
    ///         "fmt::Debug",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn qualified(self) -> Self {
        Self {
            module: self.module.qualified(),
            ..self
        }
    }

    /// Switch into a direct import mode.
    ///
    /// See [ImportMode::Direct].
    ///
    /// # Examples
    ///
    /// ```
    /// use genco::prelude::*;
    ///
    /// let ty = rust::import("std::fmt", "Debug").direct();
    ///
    /// let toks = quote!($ty);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "use std::fmt::Debug;",
    ///         "",
    ///         "Debug",
    ///     ],
    ///     toks.to_file_vec()?
    /// );
    /// # Ok::<_, genco::fmt::Error>(())
    /// ```
    pub fn direct(self) -> Self {
        Self {
            module: self.module.direct(),
            ..self
        }
    }

    /// Write the direct name of the type.
    fn write_direct(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(alias) = &self.alias {
            out.write_str(alias)
        } else {
            out.write_str(&self.name)
        }
    }

    /// Write the prefixed name of the type.
    fn write_prefixed(&self, out: &mut fmt::Formatter<'_>, module: &ItemStr) -> fmt::Result {
        if let Some(module) = module.rsplit(SEP).next() {
            out.write_str(module)?;
            out.write_str(SEP)?;
        }

        out.write_str(&self.name)?;
        Ok(())
    }
}

impl Rust {
    fn imports(out: &mut Tokens, config: &Config, tokens: &Tokens) {
        use alloc::collections::btree_set;

        use crate as genco;
        use crate::quote_in;

        let mut modules = BTreeMap::<&ItemStr, Import>::new();

        let mut queue = VecDeque::new();

        for import in tokens.walk_imports() {
            queue.push_back(import);
        }

        while let Some(import) = queue.pop_front() {
            match &import.module {
                Module::Module {
                    module,
                    import: Some(ImportMode::Direct),
                } => {
                    let module = modules.entry(module).or_default();
                    module.names.insert((&import.name, import.alias.as_ref()));
                }
                Module::Module {
                    module,
                    import: Some(ImportMode::Qualified),
                } => {
                    let module = modules.entry(module).or_default();
                    module.self_import = true;
                }
                Module::Module {
                    module,
                    import: None,
                } => match config.default_import {
                    ImportMode::Direct => {
                        let module = modules.entry(module).or_default();
                        module.names.insert((&import.name, import.alias.as_ref()));
                    }
                    ImportMode::Qualified => {
                        let module = modules.entry(module).or_default();
                        module.self_import = true;
                    }
                },
                Module::Aliased { module, alias } => {
                    let module = modules.entry(module).or_default();
                    module.self_aliases.insert(alias);
                }
            }
        }

        let mut has_any = false;

        for (m, module) in modules {
            let mut render = module.iter(m);

            if let Some(first) = render.next() {
                has_any = true;
                out.push();

                // render as a group if there's more than one thing being
                // imported.
                if let Some(second) = render.next() {
                    quote_in! { *out =>
                        use $m::{$(ref o =>
                            first.render(o);
                            quote_in!(*o => , $(ref o => second.render(o)));

                            for item in render {
                                quote_in!(*o => , $(ref o => item.render(o)));
                            }
                        )};
                    };
                } else {
                    match first {
                        RenderItem::SelfImport => {
                            quote_in!(*out => use $m;);
                        }
                        RenderItem::SelfAlias { alias } => {
                            quote_in!(*out => use $m as $alias;);
                        }
                        RenderItem::Name {
                            name,
                            alias: Some(alias),
                        } => {
                            quote_in!(*out => use $m::$name as $alias;);
                        }
                        RenderItem::Name { name, alias: None } => {
                            quote_in!(*out => use $m::$name;);
                        }
                    }
                }
            }
        }

        if has_any {
            out.line();
        }

        return;

        /// An imported module.
        #[derive(Debug, Default)]
        struct Import<'a> {
            /// If we need the module (e.g. through an alias).
            self_import: bool,
            /// Aliases for the own module.
            self_aliases: BTreeSet<&'a ItemStr>,
            /// Set of imported names.
            names: BTreeSet<(&'a ItemStr, Option<&'a ItemStr>)>,
        }

        impl<'a> Import<'a> {
            fn iter(self, module: &'a str) -> ImportedIter<'a> {
                ImportedIter {
                    module,
                    self_import: self.self_import,
                    self_aliases: self.self_aliases.into_iter(),
                    names: self.names.into_iter(),
                }
            }
        }

        struct ImportedIter<'a> {
            module: &'a str,
            self_import: bool,
            self_aliases: btree_set::IntoIter<&'a ItemStr>,
            names: btree_set::IntoIter<(&'a ItemStr, Option<&'a ItemStr>)>,
        }

        impl<'a> Iterator for ImportedIter<'a> {
            type Item = RenderItem<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                if core::mem::take(&mut self.self_import) {
                    // Only render self-import if it's not a top level module.
                    if self.module.split(SEP).count() > 1 {
                        return Some(RenderItem::SelfImport);
                    }
                }

                if let Some(alias) = self.self_aliases.next() {
                    return Some(RenderItem::SelfAlias { alias });
                }

                if let Some((name, alias)) = self.names.next() {
                    return Some(RenderItem::Name { name, alias });
                }

                None
            }
        }

        #[derive(Clone, Copy)]
        enum RenderItem<'a> {
            SelfImport,
            SelfAlias {
                alias: &'a ItemStr,
            },
            Name {
                name: &'a ItemStr,
                alias: Option<&'a ItemStr>,
            },
        }

        impl RenderItem<'_> {
            fn render(self, out: &mut Tokens) {
                match self {
                    Self::SelfImport => {
                        quote_in!(*out => self);
                    }
                    Self::SelfAlias { alias } => {
                        quote_in!(*out => self as $alias);
                    }
                    Self::Name {
                        name,
                        alias: Some(alias),
                    } => {
                        quote_in!(*out => $name as $alias);
                    }
                    Self::Name { name, alias: None } => {
                        quote_in!(*out => $name);
                    }
                }
            }
        }
    }
}

/// The import of a Rust type `use std::collections::HashMap`.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let a = rust::import("std::fmt", "Debug").qualified();
/// let b = rust::import("std::fmt", "Debug").with_module_alias("fmt2");
/// let c = rust::import("std::fmt", "Debug");
/// let d = rust::import("std::fmt", "Debug").with_alias("FmtDebug");
///
/// let toks = quote!{
///     $a
///     $b
///     $c
///     $d
/// };
///
/// assert_eq!(
///     vec![
///         "use std::fmt::{self, self as fmt2, Debug, Debug as FmtDebug};",
///         "",
///         "fmt::Debug",
///         "fmt2::Debug",
///         "Debug",
///         "FmtDebug",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// # Example with an alias
///
/// ```
/// use genco::prelude::*;
///
/// let ty = rust::import("std::fmt", "Debug").with_alias("FmtDebug");
///
/// let toks = quote!{
///     $ty
/// };
///
/// assert_eq!(
///     vec![
///         "use std::fmt::Debug as FmtDebug;",
///         "",
///         "FmtDebug",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// # Example with a module alias
///
/// ```
/// use genco::prelude::*;
///
/// let ty = rust::import("std::fmt", "Debug").with_module_alias("fmt2");
///
/// let toks = quote!{
///     $ty
/// };
///
/// assert_eq!(
///     vec![
///         "use std::fmt as fmt2;",
///         "",
///         "fmt2::Debug",
///     ],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
///
/// # Example with multiple aliases
///
/// ```
/// use genco::prelude::*;
///
/// let a = rust::import("std::fmt", "Debug").with_alias("FmtDebug");
/// let b = rust::import("std::fmt", "Debug").with_alias("FmtDebug2");
///
/// let toks = quote!{
///     $a
///     $b
/// };
///
/// assert_eq!(
///     vec![
///         "use std::fmt::{Debug as FmtDebug, Debug as FmtDebug2};",
///         "",
///         "FmtDebug",
///         "FmtDebug2",
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
        module: Module::Module {
            import: None,
            module: module.into(),
        },
        name: name.into(),
        alias: None,
    }
}
