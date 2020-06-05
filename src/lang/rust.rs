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
use std::collections::BTreeSet;
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

/// A name.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Name {
    reference: Option<Reference>,
    /// Name  of class.
    name: ItemStr,
    /// Arguments of the class.
    arguments: Vec<Type>,
}

impl Name {
    /// Format the name.
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

        out.write_str(self.name.as_ref())?;

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

    /// Add generic arguments to the given type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[feature(proc_macro_hygiene)]
    /// use genco::prelude::*;
    ///
    /// let dbg = rust::imported("std::collections", "HashMap")
    ///     .with_arguments((rust::local("T"), rust::local("U")));
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
    pub fn with_arguments(self, args: impl Args) -> Name {
        Name {
            arguments: args.into_args(),
            ..self
        }
    }

    /// Create a name with the given reference.
    pub fn reference<R: Into<Reference>>(self, reference: R) -> Name {
        Name {
            reference: Some(reference.into()),
            ..self
        }
    }
}

impl From<ItemStr> for Name {
    fn from(value: ItemStr) -> Self {
        Name {
            reference: None,
            name: value,
            arguments: vec![],
        }
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

/// An imported name in Rust.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type {
    /// Module of the imported name.
    module: Option<ItemStr>,
    /// Alias of module.
    alias: Option<ItemStr>,
    /// Name imported.
    name: Name,
    /// Qualified import.
    qualified: bool,
}

impl Type {
    fn walk_imports(&self, modules: &mut BTreeSet<(ItemStr, Option<ItemStr>)>) {
        if let Some(module) = self.module.as_ref() {
            if self.qualified || self.alias.is_some() {
                let module = ItemStr::from(format!("{}::{}", module, self.name.name.as_ref()));
                modules.insert((module, self.alias.clone()));
            } else {
                modules.insert((module.clone(), self.alias.clone()));
            }
        }

        for arg in &self.name.arguments {
            arg.walk_imports(modules);
        }
    }

    /// Alias the given type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[feature(proc_macro_hygiene)]
    /// use genco::prelude::*;
    ///
    /// let toks = quote!(#(rust::imported("std::fmt", "Debug")));
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
    pub fn alias<A: Into<ItemStr>>(self, alias: A) -> Type {
        Type {
            alias: Some(alias.into()),
            ..self
        }
    }

    /// Add generic arguments to the given type.
    pub fn with_arguments(self, args: impl Args) -> Type {
        Type {
            name: self.name.with_arguments(args),
            ..self
        }
    }

    /// Change to be a qualified import.
    pub fn qualified(self) -> Type {
        Type {
            qualified: true,
            ..self
        }
    }

    /// Make the type a reference.
    pub fn reference<R: Into<Reference>>(self, reference: R) -> Type {
        Type {
            module: self.module,
            name: self.name.reference(reference),
            alias: self.alias,
            qualified: self.qualified,
        }
    }
}

impl LangItem<Rust> for Type {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        if let Some(alias) = self.alias.as_ref() {
            out.write_str(alias)?;
            out.write_str(SEP)?;
        } else if !self.qualified {
            if let Some(part) = self.module.as_ref().and_then(|m| m.split(SEP).last()) {
                out.write_str(part)?;
                out.write_str(SEP)?;
            }
        }

        self.name.format(out, config, level)
    }

    fn as_import(&self) -> Option<&Self> {
        Some(self)
    }
}

impl Rust {
    fn imports(tokens: &Tokens) -> Option<Tokens> {
        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            import.walk_imports(&mut modules);
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for (module, alias) in modules {
            if module.split("::").count() == 1 {
                continue;
            }

            let mut s = Tokens::new();

            s.append("use ");
            s.append(module);

            if let Some(alias) = alias {
                s.append(" as ");
                s.append(alias);
            }

            s.append(";");

            out.append(s);
            out.push();
        }

        Some(out)
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

        if let Some(imports) = Self::imports(&tokens) {
            toks.append(imports);
            toks.push_line();
        }

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
/// let toks = quote!(#(rust::imported("std::fmt", "Debug")));
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
pub fn imported<M, N>(module: M, name: N) -> Type
where
    M: Into<ItemStr>,
    N: Into<ItemStr>,
{
    Type {
        module: Some(module.into()),
        alias: None,
        name: Name::from(name.into()),
        qualified: false,
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
        module: None,
        alias: None,
        name: Name::from(name.into()),
        qualified: false,
    }
}
