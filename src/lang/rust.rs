//! Specialization for Rust code generation.
//!
//! # Examples
//!
//! ```rust
//! #[feature(proc_macro_hygiene)]
//! use genco::prelude::*;
//!
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
//!     toks.to_file_vec().unwrap()
//! )
//! ```
//!
//! String quoting in Rust:
//!
//! ```rust
//! #[feature(proc_macro_hygiene)]
//! use genco::prelude::*;
//!
//! let toks: rust::Tokens = quote!(#("hello \n world".quoted()));
//! assert_eq!("\"hello \\n world\"", toks.to_string().unwrap());
//! ```

use crate::{Formatter, ItemStr, Lang, LangItem};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::{self, Write};
use std::rc::Rc;

/// Tokens container specialization for Rust.
pub type Tokens = crate::Tokens<Rust>;
/// Language box specialization for Rust.
pub type LangBox = crate::LangBox<Rust>;

impl_lang_item!(Type, Rust);
impl_plain_variadic_args!(Args, Type);

static SEP: &'static str = "::";

/// The inferred reference.
#[derive(Debug, Clone, Copy)]
pub struct Ref;

/// The static reference.
#[derive(Debug, Clone, Copy)]
pub struct StaticRef;

/// Reference information about a name.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Reference {
    /// An anonymous reference.
    Ref,
    /// A static reference.
    StaticRef,
    /// A named reference.
    Named(ItemStr),
}

impl From<Ref> for Reference {
    fn from(_: Ref) -> Self {
        Reference::Ref
    }
}

impl From<StaticRef> for Reference {
    fn from(_: StaticRef) -> Self {
        Reference::StaticRef
    }
}

impl From<Rc<String>> for Reference {
    fn from(value: Rc<String>) -> Self {
        Reference::Named(ItemStr::from(value))
    }
}

impl From<&'static str> for Reference {
    fn from(value: &'static str) -> Self {
        Reference::Named(ItemStr::Static(value))
    }
}

/// Language configuration for Rust.
#[derive(Debug)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Config {}
    }
}

#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
enum Module {
    /// Local type.
    Local,
    /// Type imported directly from module.
    Direct { module: ItemStr },
    /// Prefixed with modules own name.
    Prefixed { module: ItemStr },
    /// Prefixed with an alias.
    Aliased { module: ItemStr, alias: ItemStr },
}

impl Module {
    /// Convert into an aliased import, or keep as same in case that's not
    /// feasible.
    fn into_aliased<A>(self, alias: A) -> Self
    where
        A: Into<ItemStr>,
    {
        match self {
            Self::Direct { module } | Self::Prefixed { module } => Self::Aliased {
                module,
                alias: alias.into(),
            },
            other => other,
        }
    }

    /// Convert into a prefixed, or keep as same in case that's not feasible.
    fn into_prefixed(self) -> Self {
        match self {
            Self::Direct { module } => Self::Prefixed { module },
            other => other,
        }
    }
}

/// An imported name in Rust.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type {
    /// How the type is imported.
    module: Module,
    /// Reference information on the type.
    reference: Option<Reference>,
    /// Name of type.
    name: ItemStr,
    /// Alias to use for the type.
    alias: Option<ItemStr>,
    /// Arguments of the class.
    arguments: Vec<Type>,
}

impl Type {
    /// Alias the given type as it's imported.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[feature(proc_macro_hygiene)]
    /// use genco::prelude::*;
    ///
    /// let ty = rust::imported("std::fmt", "Debug").alias("FmtDebug");
    ///
    /// let toks = quote!(#ty);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "use std::fmt::Debug as FmtDebug;",
    ///         "",
    ///         "FmtDebug",
    ///     ],
    ///     toks.to_file_vec().unwrap()
    /// );
    /// ```
    pub fn alias<A: Into<ItemStr>>(self, alias: A) -> Self {
        Self {
            alias: Some(alias.into()),
            ..self
        }
    }

    /// Alias the module being imported.
    ///
    /// This also implies that the import is [prefixed()].
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[feature(proc_macro_hygiene)]
    /// use genco::prelude::*;
    ///
    /// let ty = rust::imported("std::fmt", "Debug").module_alias("other");
    ///
    /// let toks = quote!(#ty);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "use std::fmt as other;",
    ///         "",
    ///         "other::Debug",
    ///     ],
    ///     toks.to_file_vec().unwrap()
    /// );
    /// ```
    pub fn module_alias<A: Into<ItemStr>>(self, alias: A) -> Type {
        Type {
            module: self.module.into_aliased(alias),
            ..self
        }
    }

    /// Prefix any use of this type with the corresponding module.
    ///
    /// So importing `std::fmt::Debug` will cause the module to be referenced as
    /// `fmt::Debug` instead of `Debug`.
    ///
    /// This is implied if [module_alias()] is used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[feature(proc_macro_hygiene)]
    /// use genco::prelude::*;
    ///
    /// let ty = rust::imported("std::fmt", "Debug").prefixed();
    ///
    /// let toks = quote!(#ty);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "use std::fmt;",
    ///         "",
    ///         "fmt::Debug",
    ///     ],
    ///     toks.to_file_vec().unwrap()
    /// );
    /// ```
    pub fn prefixed(self) -> Type {
        Type {
            module: self.module.into_prefixed(),
            ..self
        }
    }

    /// Add generic arguments to the type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[feature(proc_macro_hygiene)]
    /// use genco::prelude::*;
    ///
    /// let ty = rust::imported("std::collections", "HashMap")
    ///     .with_arguments((rust::local("u32"), rust::local("u32")));
    ///
    /// let toks = quote!(#ty);
    ///
    /// assert_eq!(
    ///     vec![
    ///         "use std::collections::HashMap;",
    ///         "",
    ///         "HashMap<u32, u32>",
    ///     ],
    ///     toks.to_file_vec().unwrap()
    /// );
    /// ```
    ///
    /// ```rust
    /// #[feature(proc_macro_hygiene)]
    /// use genco::prelude::*;
    ///
    /// let dbg = rust::imported("std::collections", "HashMap")
    ///     .prefixed()
    ///     .with_arguments((rust::local("T"), rust::local("U")));
    ///
    /// let toks = quote!(#dbg);
    ///
    /// assert_eq!(
    ///     vec![
    ///        "use std::collections;",
    ///        "",
    ///        "collections::HashMap<T, U>",
    ///     ],
    ///     toks.to_file_vec().unwrap()
    /// );
    /// ```
    pub fn with_arguments(self, args: impl Args) -> Type {
        Type {
            arguments: args.into_args(),
            ..self
        }
    }

    /// Create a name with the given reference.
    pub fn reference<R: Into<Reference>>(self, reference: R) -> Self {
        Self {
            reference: Some(reference.into()),
            ..self
        }
    }
}

impl LangItem<Rust> for Type {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        if let Some(reference) = &self.reference {
            match reference {
                Reference::StaticRef => {
                    out.write_str("&'static ")?;
                }
                Reference::Named(name) => {
                    out.write_str("&'")?;
                    out.write_str(name.as_ref())?;
                    out.write_str(" ")?;
                }
                Reference::Ref => {
                    out.write_str("&")?;
                }
            }
        }

        match &self.module {
            Module::Local | Module::Direct { .. } => {
                if let Some(alias) = &self.alias {
                    out.write_str(alias)?;
                } else {
                    out.write_str(&self.name)?;
                }
            }
            Module::Prefixed { module, .. } => {
                if let Some(module) = module.rsplit("::").next() {
                    out.write_str(module)?;
                    out.write_str(SEP)?;
                }

                out.write_str(&self.name)?;
            }
            Module::Aliased {
                alias: ref module, ..
            } => {
                out.write_str(module)?;
                out.write_str(SEP)?;
                out.write_str(&self.name)?;
            }
        }

        if !self.arguments.is_empty() {
            let mut it = self.arguments.iter().peekable();

            out.write_str("<")?;

            while let Some(n) = it.next() {
                n.format(out, config, level + 1)?;

                if it.peek().is_some() {
                    out.write_str(", ")?;
                }
            }

            out.write_str(">")?;
        }

        Ok(())
    }

    fn as_import(&self) -> Option<&Self> {
        Some(self)
    }
}

impl Rust {
    fn imports(out: &mut Tokens, tokens: &Tokens) {
        use crate as genco;
        use crate::{quote, quote_in};

        let mut modules = BTreeMap::<&ItemStr, Imported>::new();

        let mut queue = VecDeque::new();

        for import in tokens.walk_imports() {
            queue.push_back(import);
        }

        while let Some(import) = queue.pop_front() {
            match &import.module {
                Module::Local => continue,
                Module::Direct { module } => {
                    let module = modules.entry(module).or_default();
                    module.names.insert((&import.name, import.alias.as_ref()));
                }
                Module::Prefixed { module } => {
                    let module = modules.entry(module).or_default();
                    module.self_import = true;
                }
                Module::Aliased { module, alias } => {
                    let module = modules.entry(module).or_default();
                    module.self_aliases.insert(alias);
                }
            }

            for arg in &import.arguments {
                queue.push_back(arg);
            }
        }

        if modules.is_empty() {
            return;
        }

        for (name, module) in modules {
            let Imported {
                self_aliases,
                self_import,
                names,
            } = module;

            let mut output = Vec::new();

            if self_import {
                output.push(quote!(self));
            }

            for alias in &self_aliases {
                output.push(quote!(self as #(*alias)));
            }

            for (name, alias) in names {
                if let Some(alias) = alias {
                    output.push(quote!(#name as #alias));
                } else {
                    output.push(quote!(#name));
                }
            }

            let mut output = output.into_iter().peekable();

            out.push();
            out.append("use");
            out.spacing();
            out.append(name);

            if let Some(item) = output.next() {
                if output.peek().is_none() {
                    if self_import {
                        out.append(";");
                        continue;
                    }

                    let mut it = self_aliases.into_iter();

                    if let (Some(first), None) = (it.next(), it.next()) {
                        out.spacing();
                        quote_in!(out => as #first;);
                        continue;
                    }

                    quote_in!(out => ::#item;);
                    continue;
                }

                out.append("::{");
                out.append(item);

                for item in output {
                    quote_in!(out => , #item);
                }

                out.append("};");
            } else {
                out.append(";");
            }
        }

        out.push_line();
        return;

        /// An imported module.
        #[derive(Default)]
        struct Imported<'a> {
            /// If we need the module (e.g. through an alias).
            self_import: bool,
            /// Aliases for the own module.
            self_aliases: BTreeSet<&'a ItemStr>,
            /// Set of imported names.
            names: BTreeSet<(&'a ItemStr, Option<&'a ItemStr>)>,
        }
    }
}

/// Language specialization for Rust.
pub struct Rust(());

impl Lang for Rust {
    type Config = Config;
    type Import = Type;

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_char('"')?;

        for c in input.chars() {
            match c {
                '\t' => out.write_str("\\t")?,
                '\n' => out.write_str("\\n")?,
                '\r' => out.write_str("\\r")?,
                '\'' => out.write_str("\\'")?,
                '"' => out.write_str("\\\"")?,
                '\\' => out.write_str("\\\\")?,
                c => out.write_char(c)?,
            };
        }

        out.write_char('"')?;
        Ok(())
    }

    fn write_file(
        tokens: Tokens,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks: Tokens = Tokens::new();

        Self::imports(&mut toks, &tokens);

        toks.extend(tokens);
        toks.format(out, config, level)
    }
}

/// Setup an imported element.
///
/// # Examples
///
/// ```rust
/// #[feature(proc_macro_hygiene)]
/// use genco::prelude::*;
///
/// let a = rust::imported("std::fmt", "Debug").prefixed();
/// let b = rust::imported("std::fmt", "Debug").module_alias("fmt2");
/// let c = rust::imported("std::fmt", "Debug");
/// let d = rust::imported("std::fmt", "Debug").alias("FmtDebug");
///
/// let toks = quote!{
///     #a
///     #b
///     #c
///     #d
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
///     toks.to_file_vec().unwrap()
/// );
/// ```
///
/// # Example with an alias
///
/// ```rust
/// #[feature(proc_macro_hygiene)]
/// use genco::prelude::*;
///
/// let ty = rust::imported("std::fmt", "Debug").alias("FmtDebug");
///
/// let toks = quote!{
///     #ty
/// };
///
/// assert_eq!(
///     vec![
///         "use std::fmt::Debug as FmtDebug;",
///         "",
///         "FmtDebug",
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// ```
///
/// # Example with a module alias
///
/// ```rust
/// #[feature(proc_macro_hygiene)]
/// use genco::prelude::*;
///
/// let ty = rust::imported("std::fmt", "Debug").module_alias("fmt2");
///
/// let toks = quote!{
///     #ty
/// };
///
/// assert_eq!(
///     vec![
///         "use std::fmt as fmt2;",
///         "",
///         "fmt2::Debug",
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// ```
///
/// # Example with multiple aliases
///
/// ```rust
/// #[feature(proc_macro_hygiene)]
/// use genco::prelude::*;
///
/// let a = rust::imported("std::fmt", "Debug").alias("FmtDebug");
/// let b = rust::imported("std::fmt", "Debug").alias("FmtDebug2");
///
/// let toks = quote!{
///     #a
///     #b
/// };
///
/// assert_eq!(
///     vec![
///         "use std::fmt::{Debug as FmtDebug, Debug as FmtDebug2};",
///         "",
///         "FmtDebug",
///         "FmtDebug2",
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// ```
pub fn imported<M, N>(module: M, name: N) -> Type
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Type {
        module: Module::Direct {
            module: module.into(),
        },
        reference: None,
        name: name.into(),
        alias: None,
        arguments: vec![],
    }
}

/// Setup a local element.
///
/// Local elements do not generate an import statement when added to a file.
///
/// # Examples
///
/// ```rust
/// #![feature(proc_macro_hygiene)]
/// use genco::prelude::*;
///
/// let toks = quote!(#(rust::local("MyType")));
/// assert_eq!(vec!["MyType"], toks.to_file_vec().unwrap());
/// ```
pub fn local<N>(name: N) -> Type
where
    N: Into<ItemStr>,
{
    Type {
        module: Module::Local,
        reference: None,
        name: name.into(),
        alias: None,
        arguments: vec![],
    }
}
