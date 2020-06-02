//! Specialization for Java code generation.

mod modifier;
mod utils;

pub use self::modifier::Modifier;
pub use self::utils::BlockComment;

use crate as genco;
use crate::{quote, Cons, Custom, Formatter};
use std::collections::{BTreeSet, HashMap};
use std::fmt::{self, Write};

/// Tokens container specialized for Java.
pub type Tokens<'el> = crate::Tokens<'el, Java<'el>>;

static JAVA_LANG: &'static str = "java.lang";
static SEP: &'static str = ".";

/// Short primitive type.
pub const SHORT: Java<'static> = Java::Primitive {
    primitive: "short",
    boxed: "Short",
};

/// Integer primitive type.
pub const INTEGER: Java<'static> = Java::Primitive {
    primitive: "int",
    boxed: "Integer",
};

/// Long primitive type.
pub const LONG: Java<'static> = Java::Primitive {
    primitive: "long",
    boxed: "Long",
};

/// Float primitive type.
pub const FLOAT: Java<'static> = Java::Primitive {
    primitive: "float",
    boxed: "Float",
};

/// Double primitive type.
pub const DOUBLE: Java<'static> = Java::Primitive {
    primitive: "double",
    boxed: "Double",
};

/// Char primitive type.
pub const CHAR: Java<'static> = Java::Primitive {
    primitive: "char",
    boxed: "Character",
};

/// Boolean primitive type.
pub const BOOLEAN: Java<'static> = Java::Primitive {
    primitive: "boolean",
    boxed: "Boolean",
};

/// Byte primitive type.
pub const BYTE: Java<'static> = Java::Primitive {
    primitive: "byte",
    boxed: "Byte",
};

/// Void (not-really) primitive type.
pub const VOID: Java<'static> = Java::Primitive {
    primitive: "void",
    boxed: "Void",
};

/// A class.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Type<'el> {
    /// Package of the class.
    package: Cons<'el>,
    /// Name  of class.
    name: Cons<'el>,
    /// Path of class when nested.
    path: Vec<Cons<'el>>,
    /// Arguments of the class.
    arguments: Vec<Java<'el>>,
}

/// An optional type.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Optional<'el> {
    /// The type that is optional.
    pub value: Box<Java<'el>>,
    /// The complete optional field type, including wrapper.
    pub field: Box<Java<'el>>,
}

/// Java token specialization.
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Java<'el> {
    /// Primitive type.
    Primitive {
        /// The boxed variant of the primitive type.
        boxed: &'static str,
        /// The primitive-primitive type.
        primitive: &'static str,
    },
    /// A class, with or without arguments, imported from somewhere.
    Class(Type<'el>),
    /// A local name with no specific qualification.
    Local {
        /// Name of class.
        name: Cons<'el>,
    },
    /// Optional type.
    Optional(Optional<'el>),
}

/// Configuration for Java formatting.
#[derive(Debug)]
pub struct Config<'el> {
    /// Package to use.
    package: Option<Cons<'el>>,

    /// Types which has been imported into the local namespace.
    imported: HashMap<String, String>,

    /// Indentation.
    indentation: usize,
}

impl crate::Config for Config<'_> {
    fn indentation(&mut self) -> usize {
        self.indentation
    }
}

impl<'el> Config<'el> {
    /// Configure package to use.
    pub fn with_package(self, package: impl Into<Cons<'el>>) -> Self {
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

impl<'el> Default for Config<'el> {
    fn default() -> Self {
        Self {
            package: Default::default(),
            imported: Default::default(),
            indentation: 4,
        }
    }
}

impl<'el> Config<'el> {
    /// Set the package name to build.
    pub fn package<P>(&mut self, package: P)
    where
        P: Into<Cons<'el>>,
    {
        self.package = Some(package.into())
    }
}

impl<'el> Java<'el> {
    /// Extend the type with a nested path.
    ///
    /// This discards any arguments associated with it.
    pub fn path<P: Into<Cons<'el>>>(&self, part: P) -> Java<'el> {
        use self::Java::*;

        match *self {
            Class(ref class) => {
                let mut path = class.path.clone();
                path.push(part.into());

                Class(Type {
                    package: class.package.clone(),
                    name: class.name.clone(),
                    path: path,
                    arguments: vec![],
                })
            }
            ref java => java.clone(),
        }
    }

    fn type_imports(java: &Java<'el>, modules: &mut BTreeSet<(Cons<'el>, Cons<'el>)>) {
        use self::Java::*;

        match *java {
            Class(ref class) => {
                for argument in &class.arguments {
                    Self::type_imports(argument, modules);
                }

                modules.insert((class.package.clone(), class.name.clone()));
            }
            _ => {}
        };
    }

    fn imports(tokens: &Tokens<'el>, config: &mut Config) -> Option<Tokens<'el>> {
        let mut modules = BTreeSet::new();

        let file_package = config.package.as_ref().map(|p| p.as_ref());

        for custom in tokens.walk_custom() {
            Self::type_imports(custom, &mut modules);
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

    /// Add arguments to the given variable.
    ///
    /// Only applies to classes, any other will return the same value.
    pub fn with_arguments(&self, arguments: Vec<Java<'el>>) -> Java<'el> {
        use self::Java::*;

        match *self {
            Class(ref cls) => Class(Type {
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
    pub fn as_raw(&self) -> Java<'el> {
        use self::Java::*;

        match *self {
            Class(ref cls) => Class(Type {
                package: cls.package.clone(),
                name: cls.name.clone(),
                path: cls.path.clone(),
                arguments: vec![],
            }),
            ref java => java.clone(),
        }
    }

    /// Get a guaranteed boxed version of a type.
    pub fn as_boxed(&self) -> Java<'el> {
        use self::Java::*;

        match *self {
            Primitive { ref boxed, .. } => Class(Type {
                package: Cons::Borrowed(JAVA_LANG),
                name: Cons::Borrowed(boxed),
                path: vec![],
                arguments: vec![],
            }),
            ref other => other.clone(),
        }
    }

    /// Compare if two types are equal.
    pub fn equals(&self, other: &Java<'el>) -> bool {
        use self::Java::*;

        match (self, other) {
            (
                &Primitive {
                    primitive: ref l_primitive,
                    ..
                },
                &Primitive {
                    primitive: ref r_primitive,
                    ..
                },
            ) => l_primitive == r_primitive,
            (&Class(ref l), &Class(ref r)) => {
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
    pub fn name(&self) -> Cons<'el> {
        use self::Java::*;

        match *self {
            Primitive { ref primitive, .. } => Cons::Borrowed(primitive),
            Class(ref cls) => cls.name.clone(),
            Local { ref name, .. } => name.clone(),
            Optional(self::Optional { ref value, .. }) => value.name(),
        }
    }

    /// Get the name of the type.
    pub fn package(&self) -> Option<Cons<'el>> {
        use self::Java::*;

        match *self {
            Primitive { .. } => Some(Cons::Borrowed(JAVA_LANG)),
            Class(ref cls) => Some(cls.package.clone()),
            Local { .. } => None,
            Optional(self::Optional { ref value, .. }) => value.package(),
        }
    }

    /// Get the arguments
    pub fn arguments(&self) -> Option<&[Java<'el>]> {
        use self::Java::*;

        match *self {
            Class(ref cls) => Some(&cls.arguments),
            Optional(self::Optional { ref value, .. }) => value.arguments(),
            _ => None,
        }
    }

    /// Check if type is optional.
    pub fn is_optional(&self) -> bool {
        use self::Java::*;

        match *self {
            Optional(_) => true,
            _ => false,
        }
    }

    /// Check if variable is primitive.
    pub fn is_primitive(&self) -> bool {
        use self::Java::*;

        match *self {
            ref p if *p == VOID => false,
            Primitive { .. } => true,
            _ => false,
        }
    }

    /// Get type as optional.
    pub fn as_optional(&self) -> Option<&Optional<'el>> {
        use self::Java::*;

        match *self {
            Optional(ref optional) => Some(optional),
            _ => None,
        }
    }

    /// Get the field type (includes optionality).
    pub fn as_field(&self) -> Java<'el> {
        self.as_optional()
            .map(|opt| (*opt.field).clone())
            .unwrap_or_else(|| self.clone())
    }

    /// Get the value type (strips optionality).
    pub fn as_value(&self) -> Java<'el> {
        self.as_optional()
            .map(|opt| (*opt.value).clone())
            .unwrap_or_else(|| self.clone())
    }

    /// Check if type is generic.
    pub fn is_generic(&self) -> bool {
        self.arguments().map(|a| !a.is_empty()).unwrap_or(false)
    }
}

impl<'el> Custom<'el> for Java<'el> {
    type Config = Config<'el>;

    fn format(&self, out: &mut Formatter, config: &mut Self::Config, level: usize) -> fmt::Result {
        use self::Java::*;

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
        tokens: Tokens<'el>,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        let mut toks: Tokens = Tokens::new();

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
pub fn imported<'a, P: Into<Cons<'a>>, N: Into<Cons<'a>>>(package: P, name: N) -> Java<'a> {
    Java::Class(Type {
        package: package.into(),
        name: name.into(),
        path: vec![],
        arguments: vec![],
    })
}

/// Setup a local element from borrowed components.
pub fn local<'el, N: Into<Cons<'el>>>(name: N) -> Java<'el> {
    Java::Local { name: name.into() }
}

/// Setup an optional type.
pub fn optional<'el, I: Into<Java<'el>>, F: Into<Java<'el>>>(value: I, field: F) -> Java<'el> {
    Java::Optional(Optional {
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
