//! Specialization for Java code generation.

mod modifier;
mod utils;

pub use self::modifier::Modifier;
pub use self::utils::BlockComment;

use crate as genco;
use crate::{quote, Cons, Formatter, Lang, LangItem};
use std::collections::{BTreeSet, HashMap};
use std::fmt;

/// Tokens container specialized for Java.
pub type Tokens<'el> = crate::Tokens<'el, Java>;

impl_lang_item!(Imported, Java);

static JAVA_LANG: &'static str = "java.lang";
static SEP: &'static str = ".";

/// Short primitive type.
pub const SHORT: Imported = Imported::Primitive {
    primitive: "short",
    boxed: "Short",
};

/// Integer primitive type.
pub const INTEGER: Imported = Imported::Primitive {
    primitive: "int",
    boxed: "Integer",
};

/// Long primitive type.
pub const LONG: Imported = Imported::Primitive {
    primitive: "long",
    boxed: "Long",
};

/// Float primitive type.
pub const FLOAT: Imported = Imported::Primitive {
    primitive: "float",
    boxed: "Float",
};

/// Double primitive type.
pub const DOUBLE: Imported = Imported::Primitive {
    primitive: "double",
    boxed: "Double",
};

/// Char primitive type.
pub const CHAR: Imported = Imported::Primitive {
    primitive: "char",
    boxed: "Character",
};

/// Boolean primitive type.
pub const BOOLEAN: Imported = Imported::Primitive {
    primitive: "boolean",
    boxed: "Boolean",
};

/// Byte primitive type.
pub const BYTE: Imported = Imported::Primitive {
    primitive: "byte",
    boxed: "Byte",
};

/// Void (not-really) primitive type.
pub const VOID: Imported = Imported::Primitive {
    primitive: "void",
    boxed: "Void",
};

/// A class.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type {
    /// Package of the class.
    package: Cons<'static>,
    /// Name  of class.
    name: Cons<'static>,
    /// Path of class when nested.
    path: Vec<Cons<'static>>,
    /// Arguments of the class.
    arguments: Vec<Imported>,
}

/// An optional type.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Optional {
    /// The type that is optional.
    pub value: Box<Imported>,
    /// The complete optional field type, including wrapper.
    pub field: Box<Imported>,
}

/// Java token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Imported {
    /// Primitive type.
    Primitive {
        /// The boxed variant of the primitive type.
        boxed: &'static str,
        /// The primitive-primitive type.
        primitive: &'static str,
    },
    /// A class, with or without arguments, imported from somewhere.
    Class(Type),
    /// A local name with no specific qualification.
    Local {
        /// Name of class.
        name: Cons<'static>,
    },
    /// Optional type.
    Optional(Optional),
}

/// Configuration for Java formatting.
#[derive(Debug)]
pub struct Config {
    /// Package to use.
    package: Option<Cons<'static>>,

    /// Types which has been imported into the local namespace.
    imported: HashMap<String, String>,

    /// Indentation.
    indentation: usize,
}

impl crate::Config for Config {
    fn indentation(&mut self) -> usize {
        self.indentation
    }
}

impl Config {
    /// Configure package to use.
    pub fn with_package(self, package: impl Into<Cons<'static>>) -> Self {
        Self {
            package: Some(package.into()),
            ..self
        }
    }

    /// Configure indentation level.
    pub fn with_indentation(self, indentation: usize) -> Self {
        Self {
            indentation,
            ..self
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            package: Default::default(),
            imported: Default::default(),
            indentation: 4,
        }
    }
}

impl Imported {
    /// Extend the type with a nested path.
    ///
    /// This discards any arguments associated with it.
    pub fn path<P: Into<Cons<'static>>>(&self, part: P) -> Imported {
        match self {
            Self::Class(class) => {
                let mut path = class.path.clone();
                path.push(part.into());

                Self::Class(Type {
                    package: class.package.clone(),
                    name: class.name.clone(),
                    path: path,
                    arguments: vec![],
                })
            }
            java => java.clone(),
        }
    }

    fn type_imports(&self, modules: &mut BTreeSet<(Cons<'static>, Cons<'static>)>) {
        match self {
            Self::Class(class) => {
                for argument in &class.arguments {
                    argument.type_imports(modules);
                }

                modules.insert((class.package.clone(), class.name.clone()));
            }
            _ => {}
        };
    }

    /// Add arguments to the given variable.
    ///
    /// Only applies to classes, any other will return the same value.
    pub fn with_arguments(&self, arguments: Vec<Imported>) -> Imported {
        match *self {
            Self::Class(ref cls) => Self::Class(Type {
                package: cls.package.clone(),
                name: cls.name.clone(),
                path: cls.path.clone(),
                arguments: arguments,
            }),
            ref java => java.clone(),
        }
    }

    /// Get the raw type.
    ///
    /// A raw type is one without generic arguments.
    pub fn as_raw(&self) -> Imported {
        match *self {
            Self::Class(ref cls) => Self::Class(Type {
                package: cls.package.clone(),
                name: cls.name.clone(),
                path: cls.path.clone(),
                arguments: vec![],
            }),
            ref java => java.clone(),
        }
    }

    /// Get a guaranteed boxed version of a type.
    pub fn as_boxed(&self) -> Imported {
        match *self {
            Self::Primitive { ref boxed, .. } => Self::Class(Type {
                package: Cons::Borrowed(JAVA_LANG),
                name: Cons::Borrowed(boxed),
                path: vec![],
                arguments: vec![],
            }),
            ref other => other.clone(),
        }
    }

    /// Compare if two types are equal.
    pub fn equals(&self, other: &Imported) -> bool {
        match (self, other) {
            (
                Self::Primitive {
                    primitive: l_primitive,
                    ..
                },
                Self::Primitive {
                    primitive: r_primitive,
                    ..
                },
            ) => l_primitive == r_primitive,
            (Self::Class(l), Self::Class(r)) => {
                l.package == r.package
                    && l.name == r.name
                    && l.arguments.len() == r.arguments.len()
                    && l.arguments
                        .iter()
                        .zip(r.arguments.iter())
                        .all(|(l, r)| l.equals(r))
            }
            _ => false,
        }
    }

    /// Get the name of the type.
    pub fn name(&self) -> Cons<'_> {
        match self {
            Self::Primitive { primitive, .. } => Cons::Borrowed(primitive),
            Self::Class(cls) => cls.name.clone(),
            Self::Local { name, .. } => name.clone(),
            Self::Optional(Optional { value, .. }) => value.name(),
        }
    }

    /// Get the name of the type.
    pub fn package(&self) -> Option<Cons<'static>> {
        match self {
            Self::Primitive { .. } => Some(Cons::Borrowed(JAVA_LANG)),
            Self::Class(cls) => Some(cls.package.clone()),
            Self::Local { .. } => None,
            Self::Optional(Optional { value, .. }) => value.package(),
        }
    }

    /// Get the arguments
    pub fn arguments(&self) -> Option<&[Imported]> {
        match self {
            Self::Class(cls) => Some(&cls.arguments),
            Self::Optional(Optional { value, .. }) => value.arguments(),
            _ => None,
        }
    }

    /// Check if type is optional.
    pub fn is_optional(&self) -> bool {
        match *self {
            Self::Optional(_) => true,
            _ => false,
        }
    }

    /// Check if variable is primitive.
    pub fn is_primitive(&self) -> bool {
        match self {
            p if *p == VOID => false,
            Self::Primitive { .. } => true,
            _ => false,
        }
    }

    /// Get type as optional.
    pub fn as_optional(&self) -> Option<&Optional> {
        match self {
            Self::Optional(optional) => Some(optional),
            _ => None,
        }
    }

    /// Get the field type (includes optionality).
    pub fn as_field(&self) -> Imported {
        self.as_optional()
            .map(|opt| (*opt.field).clone())
            .unwrap_or_else(|| self.clone())
    }

    /// Get the value type (strips optionality).
    pub fn as_value(&self) -> Imported {
        self.as_optional()
            .map(|opt| (*opt.value).clone())
            .unwrap_or_else(|| self.clone())
    }

    /// Check if type is generic.
    pub fn is_generic(&self) -> bool {
        self.arguments().map(|a| !a.is_empty()).unwrap_or(false)
    }
}

impl LangItem<Java> for Imported {
    fn format(&self, out: &mut Formatter, config: &mut Config, level: usize) -> fmt::Result {
        use self::Imported::*;

        match *self {
            Primitive {
                ref boxed,
                ref primitive,
                ..
            } => {
                if level > 0 {
                    out.write_str(boxed.as_ref())?;
                } else {
                    out.write_str(primitive.as_ref())?;
                }
            }
            Class(ref cls) => {
                {
                    let file_package = config.package.as_ref().map(|p| p.as_ref());
                    let imported = config.imported.get(cls.name.as_ref()).map(String::as_str);
                    let pkg = Some(cls.package.as_ref());

                    if cls.package.as_ref() != JAVA_LANG && imported != pkg && file_package != pkg {
                        out.write_str(cls.package.as_ref())?;
                        out.write_str(SEP)?;
                    }
                }

                {
                    out.write_str(cls.name.as_ref())?;

                    let mut it = cls.path.iter();

                    while let Some(n) = it.next() {
                        out.write_str(".")?;
                        out.write_str(n.as_ref())?;
                    }
                }

                if !cls.arguments.is_empty() {
                    out.write_str("<")?;

                    let mut it = cls.arguments.iter().peekable();

                    while let Some(argument) = it.next() {
                        argument.format(out, config, level + 1usize)?;

                        if it.peek().is_some() {
                            out.write_str(", ")?;
                        }
                    }

                    out.write_str(">")?;
                }
            }
            Local { ref name } => {
                out.write_str(name.as_ref())?;
            }
            Optional(self::Optional { ref field, .. }) => {
                field.format(out, config, level)?;
            }
        }

        Ok(())
    }

    fn as_import(&self) -> Option<&Self> {
        println!("called as_import");
        Some(self)
    }
}

/// Language specialization for Java.
pub struct Java(());

impl Java {
    fn imports<'el>(tokens: &Tokens<'el>, config: &mut Config) -> Option<Tokens<'el>> {
        let mut modules = BTreeSet::new();

        let file_package = config.package.as_ref().map(|p| p.as_ref());

        for custom in tokens.walk_custom() {
            println!("custom: {:?}", custom.as_import());

            if let Some(import) = custom.as_import() {
                import.type_imports(&mut modules);
            }
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for (package, name) in modules {
            if config.imported.contains_key(&*name) {
                continue;
            }

            if &*package == JAVA_LANG {
                continue;
            }

            if Some(&*package) == file_package.as_deref() {
                continue;
            }

            out.push(quote!(import #(package)#(SEP)#(name);));
            config
                .imported
                .insert(name.to_string(), package.to_string());
        }

        Some(out)
    }
}

impl Lang for Java {
    type Config = Config;
    type Import = Imported;

    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        use std::fmt::Write as _;

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
        let mut toks = Tokens::new();

        if let Some(ref package) = config.package {
            toks.push(toks!["package ", package.clone(), ";"]);
            toks.line_spacing();
        }

        if let Some(imports) = Self::imports(&tokens, config) {
            toks.push(imports);
            toks.line_spacing();
        }

        toks.extend(tokens);
        toks.format(out, config, level)
    }
}

/// Setup an imported element.
pub fn imported<P: Into<Cons<'static>>, N: Into<Cons<'static>>>(package: P, name: N) -> Imported {
    Imported::Class(Type {
        package: package.into(),
        name: name.into(),
        path: vec![],
        arguments: vec![],
    })
}

/// Setup a local element from borrowed components.
pub fn local<'el, N: Into<Cons<'static>>>(name: N) -> Imported {
    Imported::Local { name: name.into() }
}

/// Setup an optional type.
pub fn optional<'el, I: Into<Imported>, F: Into<Imported>>(value: I, field: F) -> Imported {
    Imported::Optional(Optional {
        value: Box::new(value.into()),
        field: Box::new(field.into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as genco;
    use crate::{quote, Java, Quoted, Tokens};

    #[test]
    fn test_primitive() {
        assert!(SHORT.is_primitive());
        assert!(INTEGER.is_primitive());
        assert!(LONG.is_primitive());
        assert!(FLOAT.is_primitive());
        assert!(DOUBLE.is_primitive());
        assert!(BOOLEAN.is_primitive());
        assert!(CHAR.is_primitive());
        assert!(BYTE.is_primitive());
        assert!(!VOID.is_primitive());
    }

    #[test]
    fn test_string() {
        let mut toks: Tokens<Java> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }

    #[test]
    fn test_imported() {
        let integer = imported("java.lang", "Integer");
        let a = imported("java.io", "A");
        let b = imported("java.io", "B");
        let ob = imported("java.util", "B");
        let ob_a = ob.with_arguments(vec![a.clone()]);

        let toks = quote!(#integer #a #b #ob #ob_a);

        assert_eq!(
            Ok("import java.io.A;\nimport java.io.B;\n\nInteger A B java.util.B java.util.B<A>\n",),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
