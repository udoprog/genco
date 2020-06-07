//! Specialization for Python code generation.
//!
//! # Examples
//!
//! String quoting in Python:
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let toks: python::Tokens = quote!(#("hello \n world".quoted()));
//! assert_eq!("\"hello \\n world\"", toks.to_string().unwrap());
//! ```

use crate::{Formatter, ItemStr, Lang, LangItem};
use std::collections::BTreeSet;
use std::fmt::{self, Write};

/// Tokens container specialization for Python.
pub type Tokens = crate::Tokens<Python>;

static SEP: &'static str = ".";

/// Python token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type {
    /// Module of the imported name.
    module: Option<ItemStr>,
    /// Alias of module.
    alias: Option<ItemStr>,
    /// Name imported.
    ///
    /// If `None`, last component of module will be used.
    name: Option<ItemStr>,
}

impl_lang_item! {
    impl FormatTokens<Python> for Type;
    impl From<Type> for LangBox<Python>;

    impl LangItem<Python> for Type {
        fn format(&self, out: &mut Formatter, _extra: &mut (), _level: usize) -> fmt::Result {
            write!(out, "{}", self)
        }

        fn as_import(&self) -> Option<&Self> {
            Some(self)
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let has_module = match self.module {
            Some(ref module) => match self.alias {
                Some(ref alias) => {
                    fmt.write_str(alias)?;
                    true
                }
                None => {
                    if let Some(part) = module.split(SEP).last() {
                        fmt.write_str(part)?;
                        true
                    } else {
                        false
                    }
                }
            },
            None => false,
        };

        if let Some(ref name) = self.name {
            if has_module {
                fmt.write_str(SEP)?;
            }

            fmt.write_str(name.as_ref())?;
        }

        Ok(())
    }
}

impl Type {
    /// Set alias for python element.
    pub fn alias<N: Into<ItemStr>>(self, new_alias: N) -> Type {
        Self {
            alias: Some(new_alias.into()),
            ..self
        }
    }

    /// Set name for python element.
    pub fn name<N: Into<ItemStr>>(self, new_name: N) -> Type {
        Self {
            name: Some(new_name.into()),
            ..self
        }
    }
}

/// Language specialization for Python.
pub struct Python(());

impl Python {
    fn imports(tokens: &Tokens) -> Option<Tokens> {
        let mut modules = BTreeSet::new();

        for import in tokens.walk_imports() {
            let Type { module, alias, .. } = import;

            if let Some(ref module) = *module {
                modules.insert((module.clone(), alias.clone()));
            }
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for (module, alias) in modules {
            let mut s = Tokens::new();

            s.append("import ");
            s.append(module);

            if let Some(alias) = alias {
                s.append(" as ");
                s.append(alias);
            }

            out.append(s);
            out.push();
        }

        Some(out)
    }
}

impl Lang for Python {
    type Config = ();
    type Import = Type;

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_char('"')?;

        for c in input.chars() {
            match c {
                '\t' => out.write_str("\\t")?,
                '\u{0007}' => out.write_str("\\b")?,
                '\n' => out.write_str("\\n")?,
                '\r' => out.write_str("\\r")?,
                '\u{0014}' => out.write_str("\\f")?,
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
        let mut toks = Tokens::new();

        if let Some(imports) = Self::imports(&tokens) {
            toks.append(imports);
            toks.line();
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
/// use genco::prelude::*;
///
/// let toks = quote! {
///     #(python::imported("collections").name("named_tuple"))
///     #(python::imported("collections"))
///     #(python::imported("collections").alias("c").name("named_tuple"))
///     #(python::imported("collections").alias("c"))
/// };
///
/// assert_eq!(
///     vec![
///         "import collections",
///         "import collections as c",
///         "",
///         "collections.named_tuple",
///         "collections",
///         "c.named_tuple",
///         "c",
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// ```
pub fn imported<M>(module: M) -> Type
where
    M: Into<ItemStr>,
{
    Type {
        module: Some(module.into()),
        alias: None,
        name: None,
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
/// let toks = quote!(#(python::local("dict")));
/// assert_eq!(vec!["dict"], toks.to_file_vec().unwrap());
/// ```
pub fn local<N>(name: N) -> Type
where
    N: Into<ItemStr>,
{
    Type {
        module: None,
        alias: None,
        name: Some(name.into()),
    }
}
