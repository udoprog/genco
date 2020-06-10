//! Specialization for Rust code generation.
//!
//! # Examples
//!
//! ```rust
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

impl_plain_variadic_args!(Args, Type);

/// The `()` (unit) type.
pub const UNIT: Type = const_local("()");
/// The `!` (never) type.
pub const NEVER: Type = const_local("!");
/// The `u8` type.
pub const U8: Type = const_local("u8");
/// The `u16` type.
pub const U16: Type = const_local("u16");
/// The `u32` type.
pub const U32: Type = const_local("u32");
/// The `u64` type.
pub const U64: Type = const_local("u64");
/// The `u128` type.
pub const U128: Type = const_local("u128");
/// The `i8` type.
pub const I8: Type = const_local("i8");
/// The `i16` type.
pub const I16: Type = const_local("i16");
/// The `i32` type.
pub const I32: Type = const_local("i32");
/// The `i64` type.
pub const I64: Type = const_local("i64");
/// The `i128` type.
pub const I128: Type = const_local("i128");
/// The `usize` type.
pub const USIZE: Type = const_local("usize");
/// The `isize` type.
pub const ISIZE: Type = const_local("isize");

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
pub struct Config {
    default_import: Import,
}

impl Config {
    /// Configure the default import policy to use.
    ///
    /// See [Import] for more details.
    pub fn with_default_import(self, default_import: Import) -> Self {
        Self {
            default_import,
            ..self
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_import: Import::Direct,
        }
    }
}

/// The import policy to use when generating import statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Import {
    /// Import names without a module prefix.
    ///
    /// so for `std::fmt::Debug` it would import `std::fmt::Debug`, and use
    /// `Debug`.
    Direct,
    /// Import names with a module prefix.
    ///
    /// so for `std::fmt::Debug` it would import `std::fmt`, and use
    /// `fmt::Debug`.
    Prefixed,
}

#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
enum Module {
    /// Local type.
    Local,
    /// Type imported directly from module with the specified policy.
    Module {
        import: Option<Import>,
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
                import: Some(Import::Direct),
                module,
            },
            other => other,
        }
    }

    /// Convert into a prefixed, or keep as same in case that's not feasible.
    fn into_prefixed(self) -> Self {
        match self {
            Self::Module { module, .. } => Self::Module {
                module,
                import: Some(Import::Prefixed),
            },
            other => other,
        }
    }
}

/// An imported name in Rust.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type {
    /// Reference information on the type.
    reference: Option<Reference>,
    /// If the type is dynamic.
    dyn_type: bool,
    /// How the type is imported.
    module: Module,
    /// Name of type.
    name: ItemStr,
    /// Arguments of the class.
    arguments: Vec<Type>,
    /// Alias to use for the type.
    alias: Option<ItemStr>,
}

impl Type {
    /// Alias the given type as it's imported.
    ///
    /// # Examples
    ///
    /// ```rust
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
            module: self.module.into_aliased(),
            alias: Some(alias.into()),
            ..self
        }
    }

    /// Alias the module being imported.
    ///
    /// This also implies that the import is [prefixed].
    ///
    /// # Examples
    ///
    /// ```rust
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
    ///
    /// [prefixed]: Self::prefixed()
    pub fn module_alias<A: Into<ItemStr>>(self, alias: A) -> Type {
        Type {
            module: self.module.into_module_aliased(alias),
            ..self
        }
    }

    /// Prefix any use of this type with the corresponding module.
    ///
    /// So importing `std::fmt::Debug` will cause the module to be referenced as
    /// `fmt::Debug` instead of `Debug`.
    ///
    /// This is implied if [module_alias()][Self::module_alias()] is used.
    ///
    /// # Examples
    ///
    /// ```rust
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

    /// Convert into a reference `&<type>` type.
    pub fn reference<R: Into<Reference>>(self, reference: R) -> Self {
        Self {
            reference: Some(reference.into()),
            ..self
        }
    }

    /// Convert into a dynamic `dyn <type>` type.
    pub fn into_dyn(self) -> Self {
        Self {
            dyn_type: true,
            ..self
        }
    }

    /// Write the direct name of the type.
    fn write_direct(&self, out: &mut Formatter) -> fmt::Result {
        if let Some(alias) = &self.alias {
            out.write_str(alias)
        } else {
            out.write_str(&self.name)
        }
    }

    /// Write the prefixed name of the type.
    fn write_prefixed(&self, out: &mut Formatter, module: &ItemStr) -> fmt::Result {
        if let Some(module) = module.rsplit("::").next() {
            out.write_str(module)?;
            out.write_str(SEP)?;
        }

        out.write_str(&self.name)?;
        Ok(())
    }
}

impl_lang_item! {
    impl FormatTokens<Rust> for Type;
    impl From<Type> for LangBox<Rust>;

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

            if self.dyn_type {
                out.write_str("dyn ")?;
            }

            match &self.module {
                Module::Local
                | Module::Module {
                    import: Some(Import::Direct),
                    ..
                } => {
                    self.write_direct(out)?;
                }
                Module::Module {
                    import: Some(Import::Prefixed),
                    module,
                } => {
                    self.write_prefixed(out, module)?;
                }
                Module::Module {
                    import: None,
                    module,
                } => match &config.default_import {
                    Import::Direct => self.write_direct(out)?,
                    Import::Prefixed => self.write_prefixed(out, module)?,
                },
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
}

impl Rust {
    fn imports(out: &mut Tokens, config: &mut Config, tokens: &Tokens) {
        use crate as genco;
        use crate::quote_in;
        use std::collections::btree_set;

        let mut modules = BTreeMap::<&ItemStr, Imported>::new();

        let mut queue = VecDeque::new();

        for import in tokens.walk_imports() {
            queue.push_back(import);
        }

        while let Some(import) = queue.pop_front() {
            match &import.module {
                Module::Local => continue,
                Module::Module {
                    module,
                    import: Some(Import::Direct),
                } => {
                    let module = modules.entry(module).or_default();
                    module.names.insert((&import.name, import.alias.as_ref()));
                }
                Module::Module {
                    module,
                    import: Some(Import::Prefixed),
                } => {
                    let module = modules.entry(module).or_default();
                    module.self_import = true;
                }
                Module::Module {
                    module,
                    import: None,
                } => match config.default_import {
                    Import::Direct => {
                        let module = modules.entry(module).or_default();
                        module.names.insert((&import.name, import.alias.as_ref()));
                    }
                    Import::Prefixed => {
                        let module = modules.entry(module).or_default();
                        module.self_import = true;
                    }
                },
                Module::Aliased { module, alias } => {
                    let module = modules.entry(module).or_default();
                    module.self_aliases.insert(alias);
                }
            }

            for arg in &import.arguments {
                queue.push_back(arg);
            }
        }

        let mut has_any = false;

        for (m, module) in modules {
            let mut render = module.iter();

            if let Some(first) = render.next() {
                has_any = true;
                out.push();

                // render as a group if there's more than one thing being
                // imported.
                if let Some(second) = render.next() {
                    quote_in! { *out =>
                        use #m::{#( o =>
                            first.render(o);
                            quote_in!(*o => , #(o => second.render(o)));

                            for item in render {
                                quote_in!(*o => , #(o => item.render(o)));
                            }
                        )};
                    };
                } else {
                    match first {
                        RenderItem::SelfImport => {
                            quote_in!(*out => use #m;);
                        }
                        RenderItem::SelfAlias { alias } => {
                            quote_in!(*out => use #m as #alias;);
                        }
                        RenderItem::Name {
                            name,
                            alias: Some(alias),
                        } => {
                            quote_in!(*out => use #m::#name as #alias;);
                        }
                        RenderItem::Name { name, alias: None } => {
                            quote_in!(*out => use #m::#name;);
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
        struct Imported<'a> {
            /// If we need the module (e.g. through an alias).
            self_import: bool,
            /// Aliases for the own module.
            self_aliases: BTreeSet<&'a ItemStr>,
            /// Set of imported names.
            names: BTreeSet<(&'a ItemStr, Option<&'a ItemStr>)>,
        }

        impl<'a> Imported<'a> {
            fn iter(self) -> ImportedIter<'a> {
                ImportedIter {
                    self_import: self.self_import,
                    self_aliases: self.self_aliases.into_iter(),
                    names: self.names.into_iter(),
                }
            }
        }

        struct ImportedIter<'a> {
            self_import: bool,
            self_aliases: btree_set::IntoIter<&'a ItemStr>,
            names: btree_set::IntoIter<(&'a ItemStr, Option<&'a ItemStr>)>,
        }

        impl<'a> Iterator for ImportedIter<'a> {
            type Item = RenderItem<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                if std::mem::take(&mut self.self_import) {
                    return Some(RenderItem::SelfImport);
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
                        quote_in!(*out => self as #alias);
                    }
                    Self::Name {
                        name,
                        alias: Some(alias),
                    } => {
                        quote_in!(*out => #name as #alias);
                    }
                    Self::Name { name, alias: None } => {
                        quote_in!(*out => #name);
                    }
                }
            }
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

        Self::imports(&mut toks, config, &tokens);

        toks.extend(tokens);
        toks.format(out, config, level)
    }
}

/// Setup an imported element.
///
/// # Examples
///
/// ```rust
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
        reference: None,
        dyn_type: false,
        module: Module::Module {
            import: None,
            module: module.into(),
        },
        name: name.into(),
        arguments: vec![],
        alias: None,
    }
}

/// Setup a local element.
///
/// Local elements do not generate an import statement when added to a file.
///
/// # Examples
///
/// ```rust
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
        dyn_type: false,
        name: name.into(),
        arguments: vec![],
        alias: None,
    }
}

/// Helper function to construct a constant local type.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// const MY_TYPE: rust::Type = rust::const_local("MyType");
/// ```
pub const fn const_local(name: &'static str) -> Type {
    Type {
        module: Module::Local,
        reference: None,
        dyn_type: false,
        name: ItemStr::Static(name),
        arguments: Vec::new(),
        alias: None,
    }
}
