//! Specialization for Java code generation.

use super::custom::Custom;
use super::formatter::Formatter;
use std::fmt;
use std::borrow::Cow;
use super::tokens::Tokens;
use std::collections::{HashMap, BTreeSet};

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

/// Java token specialization.
#[derive(Debug, Clone)]
pub enum Java<'el> {
    /// Primitive type.
    Primitive {
        /// The boxed variant of the primitive type.
        boxed: &'static str,
        /// The primitive-primitive type.
        primitive: &'static str,
    },
    /// A class, with or without arguments, imported from somewhere.
    Class {
        /// Package of the class.
        package: Cow<'el, str>,
        /// Name of class.
        name: Cow<'el, str>,
        /// Arguments of the class.
        arguments: Vec<Java<'el>>,
    },
}

/// Extra data for Java formatting.
#[derive(Debug, Default)]
pub struct JavaExtra {
    /// Types which has been imported into the local namespace.
    imported: HashMap<String, String>,
}

impl<'el> Java<'el> {
    fn type_imports<'a>(java: &'a Java<'a>, modules: &mut BTreeSet<(&'a str, &'a str)>) {
        use self::Java::*;

        match *java {
            Class {
                ref package,
                ref name,
                ref arguments,
            } => {
                for argument in arguments {
                    Self::type_imports(argument, modules);
                }

                modules.insert((package.as_ref(), name.as_ref()));
            }
            _ => {}
        };
    }

    fn imports<'a>(
        tokens: &'a Tokens<'a, Self>,
        extra: &mut JavaExtra,
    ) -> Option<Tokens<'a, Self>> {
        let mut modules = BTreeSet::new();

        for custom in tokens.walk_custom() {
            Self::type_imports(custom, &mut modules);
        }

        if modules.is_empty() {
            return None;
        }

        let mut out = Tokens::new();

        for (package, name) in modules {
            if extra.imported.contains_key(name) {
                continue;
            }

            if package == JAVA_LANG {
                continue;
            }

            out.push(toks!("import ", package, SEP, name, ";"));
            extra.imported.insert(name.to_string(), package.to_string());
        }

        Some(out)
    }

    /// Add arguments to the given variable.
    ///
    /// Only applies to classes, any other will return the same value.
    pub fn with_arguments(&self, arguments: Vec<Java<'el>>) -> Java<'el> {
        use self::Java::*;

        match *self {
            Class {
                ref package,
                ref name,
                ..
            } => {
                Class {
                    package: package.clone(),
                    name: name.clone(),
                    arguments: arguments,
                }
            }
            ref java => java.clone(),
        }
    }

    /// Compare if two types are equal.
    pub fn equals(&self, other: &Java) -> bool {
        use self::Java::*;

        match (self, other) {
            (&Primitive { primitive: ref l_primitive, .. },
             &Primitive { primitive: ref r_primitive, .. }) => l_primitive == r_primitive,
            (&Class {
                 package: ref l_package,
                 name: ref l_name,
                 arguments: ref l_arguments,
             },
             &Class {
                 package: ref r_package,
                 name: ref r_name,
                 arguments: ref r_arguments,
             }) => {
                l_package == r_package && l_name == r_name &&
                    l_arguments.len() == r_arguments.len() &&
                    l_arguments.iter().zip(r_arguments.iter()).all(|(l, r)| {
                        l.equals(r)
                    })
            }
            _ => false,
        }
    }
}

impl<'el> Custom for Java<'el> {
    type Extra = JavaExtra;

    fn format(&self, out: &mut Formatter, extra: &mut Self::Extra, level: usize) -> fmt::Result {
        use self::Java::*;

        match *self {
            Primitive {
                ref boxed,
                ref primitive,
                ..
            } => {
                if level > 1 {
                    out.write_str(boxed.as_ref())?;
                } else {
                    out.write_str(primitive.as_ref())?;
                }
            }
            Class {
                ref package,
                ref name,
                ref arguments,
                ..
            } => {
                if package != JAVA_LANG &&
                    extra.imported.get(name.as_ref()).map(String::as_str) !=
                        Some(package.as_ref())
                {
                    out.write_str(package.as_ref())?;
                    out.write_str(SEP)?;
                }

                out.write_str(name.as_ref())?;

                if !arguments.is_empty() {
                    out.write_str("<")?;

                    for argument in arguments {
                        argument.format(out, extra, level + 1usize)?;
                    }

                    out.write_str(">")?;
                }
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

    fn write_file<'a>(
        tokens: Tokens<'a, Self>,
        out: &mut Formatter,
        extra: &mut Self::Extra,
        level: usize,
    ) -> fmt::Result {
        let mut toks: Tokens<Self> = Tokens::new();

        if let Some(imports) = Self::imports(&tokens, extra) {
            toks.push(imports);
        }

        toks.push_ref(&tokens);
        toks.join_line_spacing().format(out, extra, level)
    }
}

/// Setup an imported element.
pub fn imported<'a>(package: Cow<'a, str>, name: Cow<'a, str>) -> Java<'a> {
    Java::Class {
        package: package,
        name: name,
        arguments: vec![],
    }
}

/// Setup an imported element from borrowed components.
pub fn imported_ref<'a>(package: &'a str, name: &'a str) -> Java<'a> {
    Java::Class {
        package: Cow::Borrowed(package),
        name: Cow::Borrowed(name),
        arguments: vec![],
    }
}

#[cfg(test)]
mod tests {
    use tokens::Tokens;
    use java::Java;
    use quoted::Quoted;
    use super::imported_ref;

    #[test]
    fn test_string() {
        let mut toks: Tokens<Java> = Tokens::new();
        toks.append("hello \n world".quoted());
        assert_eq!("\"hello \\n world\"", toks.to_string().unwrap().as_str());
    }

    #[test]
    fn test_imported() {
        let integer = imported_ref("java.lang", "Integer");
        let a = imported_ref("java.io", "A");
        let b = imported_ref("java.io", "B");
        let ob = imported_ref("java.util", "B");
        let ob_a = ob.with_arguments(vec![a.clone()]);

        let toks = toks!(integer, a, b, ob, ob_a).join_spacing();

        assert_eq!(
            Ok(
                "import java.io.A;\nimport java.io.B;\n\nInteger A B java.util.B java.util.B<A>\n",
            ),
            toks.to_file().as_ref().map(|s| s.as_str())
        );
    }
}
