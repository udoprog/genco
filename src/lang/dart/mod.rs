//! Specialization for Dart code generation.

mod modifier;
mod utils;

pub use self::modifier::Modifier;
pub use self::utils::DocComment;

use crate::{Cons, Formatter, Lang, LangItem};
use std::fmt::{self, Write};
use std::rc::Rc;

/// Tokens container specialization for Dart.
pub type Tokens<'el> = crate::Tokens<'el, Dart>;

/// Trait implemented by dart types.
pub trait DartType: LangItem<Dart> {}

impl_lang_item!(Type, Dart);
impl_lang_item!(BuiltIn, Dart);
impl_lang_item!(Local, Dart);
impl_lang_item!(Void, Dart);
impl_lang_item!(Dynamic, Dart);

impl DartType for Type {}
impl DartType for BuiltIn {}
impl DartType for Local {}
impl DartType for Void {}
impl DartType for Dynamic {}

static SEP: &'static str = ".";

/// dart:core package.
pub static DART_CORE: &'static str = "dart:core";

/// The type corresponding to `void`.
pub const VOID: Void = Void(());

/// The type corresponding to `dynamic`.
pub const DYNAMIC: Dynamic = Dynamic(());

/// Integer built-in type.
pub const INT: BuiltIn = BuiltIn { name: "int" };

/// Double built-in type.
pub const DOUBLE: BuiltIn = BuiltIn { name: "double" };

/// Boolean built-in type.
pub const BOOL: BuiltIn = BuiltIn { name: "bool" };

/// Config data for Dart formatting.
#[derive(Debug, Default)]
pub struct Config {}

/// built-in types.
#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct BuiltIn {
    /// The built-in type.
    name: &'static str,
}

impl LangItem<Dart> for BuiltIn {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str(self.name)
    }
}

/// a locally defined type.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Local {
    name: Cons<'static>,
}

impl LangItem<Dart> for Local {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str(&*self.name)
    }
}

/// the void type.
#[derive(Clone, Copy)]
pub struct Void(());

impl LangItem<Dart> for Void {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str("void")
    }
}

/// The dynamic type.
#[derive(Clone, Copy)]
pub struct Dynamic(());

impl LangItem<Dart> for Dynamic {
    fn format(&self, out: &mut Formatter, _: &mut Config, _: usize) -> fmt::Result {
        out.write_str("dynamic")
    }
}

/// A custom dart type.
#[derive(Clone)]
pub struct Type {
    /// Path to import.
    path: Cons<'static>,
    /// Name imported.
    name: Cons<'static>,
    /// Alias of module.
    alias: Option<Cons<'static>>,
    /// Generic arguments.
    arguments: Vec<Rc<dyn DartType>>,
}

impl Type {
    /// Add an `as` keyword to the import.
    pub fn alias(self, alias: impl Into<Cons<'static>>) -> Type {
        Self {
            alias: Some(alias.into()),
            ..self
        }
    }

    /// Add arguments to the given variable.
    ///
    /// Only applies to classes, any other will return the same value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #![feature(proc_macro_hygiene)]
    ///
    /// use genco::prelude::*;
    /// use genco::dart::*;
    ///
    /// let ty = imported("dart:collection", "Map").with_arguments((INT, VOID));
    ///
    /// assert_eq!("import \"dart:collection\";\n\nMap<int, void>\n", quote!(#ty).to_file_string().unwrap());
    /// ```
    pub fn with_arguments(self, args: impl Args) -> Type {
        Self {
            arguments: args.into_args(),
            ..self
        }
    }

    /// Convert into raw type.
    pub fn raw(self) -> Type {
        Self {
            arguments: vec![],
            ..self
        }
    }

    /// Check if this type belongs to a core package.
    pub fn is_core(&self) -> bool {
        &*self.path != DART_CORE
    }

    /// Check if type is generic.
    pub fn is_generic(&self) -> bool {
        !self.arguments.is_empty()
    }
}

impl LangItem<Dart> for Type {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        if let Some(alias) = &self.alias {
            out.write_str(alias.as_ref())?;
            out.write_str(SEP)?;
        }

        out.write_str(&*self.name)?;

        if !self.arguments.is_empty() {
            out.write_str("<")?;

            let mut it = self.arguments.iter().peekable();

            while let Some(argument) = it.next() {
                argument.format(out, config, level + 1)?;

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

/// Language specialization for Dart.
pub struct Dart(());

impl Dart {
    /// Resolve all imports.
    fn imports<'el>(input: &Tokens<'el>, _: &mut Config) -> Tokens<'el> {
        use crate::quoted::Quoted;
        use std::collections::BTreeSet;

        let mut modules = BTreeSet::new();

        for custom in input.walk_custom() {
            if let Some(ty) = custom.as_import() {
                if &*ty.path == DART_CORE {
                    continue;
                }

                modules.insert((ty.path.clone(), ty.alias.clone()));
            }
        }

        if modules.is_empty() {
            return toks!();
        }

        let mut o = toks!();

        for (name, alias) in modules {
            if let Some(alias) = alias {
                o.push(toks!("import ", name.quoted(), " as ", alias, ";"));
            } else {
                o.push(toks!("import ", name.quoted(), ";"));
            }
        }

        return o;
    }
}

impl Lang for Dart {
    type Config = Config;
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
            }
        }

        out.write_char('"')?;

        Ok(())
    }

    fn write_file(
        tokens: Tokens<'_>,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks: Tokens = Tokens::new();

        let imports = Self::imports(&tokens, config);

        if !imports.is_empty() {
            toks.push(imports);
            toks.line_spacing();
        }

        toks.append(tokens);
        toks.format(out, config, level)
    }
}

/// Setup an imported element.
pub fn imported<P: Into<Cons<'static>>, N: Into<Cons<'static>>>(path: P, name: N) -> Type {
    Type {
        path: path.into(),
        alias: None,
        name: name.into(),
        arguments: Vec::new(),
    }
}

/// Setup a local element.
pub fn local<N: Into<Cons<'static>>>(name: N) -> Local {
    Local { name: name.into() }
}

/// Helper trait for things that can be turned into generic arguments.
pub trait Args {
    /// Convert the given type into a collection of arguments.
    fn into_args(self) -> Vec<Rc<dyn DartType>>;
}

impl<T> Args for T
where
    T: 'static + DartType,
{
    fn into_args(self) -> Vec<Rc<dyn DartType>> {
        vec![Rc::new(self)]
    }
}

macro_rules! impl_args {
    ($($ident:ident => $var:ident),*) => {
        impl<$($ident,)*> Args for ($($ident,)*) where $($ident: 'static + DartType,)* {
            fn into_args(self) -> Vec<Rc<dyn DartType>> {
                let ($($var,)*) = self;
                vec![$(Rc::new($var),)*]
            }
        }
    }
}

impl_args!(A => a);
impl_args!(A => a, B => b);
impl_args!(A => a, B => b, C => c);
impl_args!(A => a, B => b, C => c, D => d);
impl_args!(A => a, B => b, C => c, D => d, E => e);

#[cfg(test)]
mod tests {
    use super::*;
    use crate as genco;
    use crate::{quote, Quoted};

    #[test]
    fn test_builtin() {
        assert_eq!("int", quote!(#INT).to_string().unwrap());
        assert_eq!("double", quote!(#DOUBLE).to_string().unwrap());
        assert_eq!("bool", quote!(#BOOL).to_string().unwrap());
        // assert!(!VOID.is_built_in());
    }

    #[test]
    fn test_string() {
        let mut toks = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap());
    }

    #[test]
    fn test_imported() {
        let a = imported("package:http/http.dart", "A");
        let b = imported("package:http/http.dart", "B");
        let c = imported("package:http/http.dart", "C").alias("h2");
        let d = imported("../http.dart", "D");

        let toks = quote!(#a #b #c #d);

        let expected = vec![
            "import \"../http.dart\";",
            "import \"package:http/http.dart\";",
            "import \"package:http/http.dart\" as h2;",
            "",
            "A B h2.C D",
            "",
        ];

        assert_eq!(expected, toks.to_file_vec().unwrap());
    }
}
