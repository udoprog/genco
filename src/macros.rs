//! Macros in GenCo

/// Helper macro to reduce boilerplate needed with nested token expressions.
///
/// ## Examples
///
/// ```rust,ignore
/// # #![allow(deprecated)]
/// let n1: genco::Tokens<()> = toks!("var v = ", "bar".quoted(), ";");
/// ```
#[macro_export]
#[deprecated(since = "0.5.0", note = "Use the quote! procedural macro instead.")]
macro_rules! toks {
    ($($x:expr),*) => {
        {
            let mut _t = $crate::Tokens::new();
            $(_t.append($x);)*
            _t
        }
    };

    ($($x:expr,)*) => {toks!($($x),*)}
}

/// Helper macro to reduce boilerplate needed with pushed token expressions.
///
/// All arguments being pushed are cloned, which should be cheap for reference types.
///
/// ## Examples
///
/// ```rust
/// # #![allow(deprecated)]
/// # use genco::push;
/// # fn main() {
/// use genco::{Tokens, Java, Cons};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = Cons::from(String::from("hello"));
///
/// push!(toks, "foo ", id);
/// push!(toks, "bar ", id);
///
/// let mut out = Vec::new();
/// out.push("foo hello");
/// out.push("bar hello");
///
/// assert_eq!(out.join("\n").as_str(), toks.to_string().unwrap().as_str());
/// # }
/// ```
///
/// Pushing as a block:
///
/// ```rust
/// # #[macro_use] extern crate genco;
/// # fn main() {
/// use genco::{Tokens, Java, Cons};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = Cons::from(String::from("hello"));
///
/// push!(toks, |t| {
///   push!(t, "foo ", id);
///   push!(t, "bar ", id);
/// });
///
/// let mut out = Vec::new();
/// out.push("foo hello");
/// out.push("bar hello");
///
/// assert_eq!(out.join("\n").as_str(), toks.to_string().unwrap().as_str());
/// # }
/// ```
#[macro_export]
#[deprecated(since = "0.5.0", note = "Use the quote! procedural macro instead.")]
macro_rules! push {
    ($dest:expr, |$t:ident| $code:block) => {
        $dest.push({
            let mut $t = $crate::Tokens::new();
            $code
            $t
        })
    };

    ($dest:expr, $($x:expr),*) => {
        $dest.push({
            let mut _t = $crate::Tokens::new();
            $(_t.append(Clone::clone(&$x));)*
            _t
        })
    };

    ($dest:expr, $($x:expr,)*) => {push!($dest, $($x),*)};
}

/// Helper macro to reduce boilerplate needed with nested token expressions.
///
/// All arguments being pushed are cloned, which should be cheap for reference types.
///
/// ## Examples
///
/// ```rust
/// # #![allow(deprecated)]
/// # use genco::nested;
/// # fn main() {
/// use genco::{Tokens, Java, Cons};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = Cons::from(String::from("hello"));
///
/// nested!(toks, "foo ", id);
/// nested!(toks, "bar ", id);
///
/// let mut out = Vec::new();
/// out.push("    foo hello");
/// out.push("    bar hello");
/// out.push("");
///
/// assert_eq!(out.join("\n").as_str(), toks.to_string().unwrap().as_str());
/// # }
/// ```
///
/// Pushing as a block:
///
/// ```rust
/// # #![allow(deprecated)]
/// # use genco::{nested, push};
/// # fn main() {
/// use genco::{Tokens, Java, Cons};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = Cons::from(String::from("hello"));
///
/// nested!(toks, |t| {
///   push!(t, "foo ", id);
///   push!(t, "bar ", id);
/// });
///
/// let mut out = Vec::new();
/// out.push("    foo hello");
/// out.push("    bar hello");
/// out.push("");
///
/// assert_eq!(out.join("\n").as_str(), toks.to_string().unwrap().as_str());
/// # }
/// ```
#[macro_export]
#[deprecated(since = "0.5.0", note = "Use the quote! procedural macro instead.")]
macro_rules! nested {
    ($dest:expr, |$t:ident| $code:block) => {
        $dest.nested({
            let mut $t = $crate::Tokens::new();
            $code
            $t
        })
    };

    ($dest:expr, $($x:expr),*) => {
        $dest.nested({
            let mut _t = $crate::Tokens::new();
            $(_t.append(Clone::clone(&$x));)*
            _t
        })
    };

    ($dest:expr, $($x:expr,)*) => {nested!($dest, $($x),*)};
}

macro_rules! impl_lang_item {
    ($ty:ident, $lang:ty) => {
        impl<'el> crate::FormatTokens<'el, $lang> for $ty {
            fn format_tokens(self, tokens: &mut crate::Tokens<'el, $lang>) {
                tokens.elements.push(crate::Element::LangBox(self.into()));
            }
        }

        impl<'el> crate::FormatTokens<'el, $lang> for &'el $ty {
            fn format_tokens(self, tokens: &mut crate::Tokens<'el, $lang>) {
                tokens.elements.push(crate::Element::LangBox(self.into()));
            }
        }

        impl<'el> From<$ty> for crate::LangBox<'el, $lang> {
            fn from(value: $ty) -> Self {
                use std::rc::Rc;
                crate::LangBox::Rc(Rc::new(value) as Rc<dyn LangItem<$lang>>)
            }
        }

        impl<'el> From<&'el $ty> for crate::LangBox<'el, $lang> {
            fn from(value: &'el $ty) -> Self {
                crate::LangBox::Ref(value)
            }
        }
    };
}

macro_rules! impl_variadic_type_args {
    ($args:ident, $ty:ident, $type_box:ident) => {
        /// Helper trait for things that can be turned into generic arguments.
        pub trait $args {
            /// Convert the given type into a collection of arguments.
            fn into_args(self) -> Vec<$type_box>;
        }

        impl<T> $args for T
        where
            T: 'static + $ty,
        {
            fn into_args(self) -> Vec<$type_box> {
                vec![$type_box::new(self)]
            }
        }

        impl_variadic_type_args!(@args $args, $ty, $type_box, A => a);
        impl_variadic_type_args!(@args $args, $ty, $type_box, A => a, B => b);
        impl_variadic_type_args!(@args $args, $ty, $type_box, A => a, B => b, C => c);
        impl_variadic_type_args!(@args $args, $ty, $type_box, A => a, B => b, C => c, D => d);
        impl_variadic_type_args!(@args $args, $ty, $type_box, A => a, B => b, C => c, D => d, E => e);
    };

    (@args $args:ty, $ty:ident, $type_box:ident, $($ident:ident => $var:ident),*) => {
        impl<$($ident,)*> $args for ($($ident,)*) where $($ident: 'static + $ty,)* {
            fn into_args(self) -> Vec<$type_box> {
                let ($($var,)*) = self;
                vec![$($type_box::new($var),)*]
            }
        }
    };
}

macro_rules! impl_plain_variadic_args {
    ($args:ident, $ty:ident) => {
        /// Helper trait for things that can be turned into generic arguments.
        pub trait $args {
            /// Convert the given type into a collection of arguments.
            fn into_args(self) -> Vec<$ty>;
        }

        impl<T> $args for T
        where
            T: Into<$ty>,
        {
            fn into_args(self) -> Vec<$ty> {
                vec![self.into()]
            }
        }

        impl_plain_variadic_args!(@args $args, $ty, A => a);
        impl_plain_variadic_args!(@args $args, $ty, A => a, B => b);
        impl_plain_variadic_args!(@args $args, $ty, A => a, B => b, C => c);
        impl_plain_variadic_args!(@args $args, $ty, A => a, B => b, C => c, D => d);
        impl_plain_variadic_args!(@args $args, $ty, A => a, B => b, C => c, D => d, E => e);
    };

    (@args $args:ty, $ty:ident, $($ident:ident => $var:ident),*) => {
        impl<$($ident,)*> $args for ($($ident,)*) where $($ident: Into<$ty>,)* {
            fn into_args(self) -> Vec<$ty> {
                let ($($var,)*) = self;
                vec![$($var.into(),)*]
            }
        }
    };
}

macro_rules! impl_type_basics {
    ($lang:ty, $enum:ident<$lt:lifetime>, $trait:ident, $type_box:ident, $args:ident, {$($ty:ident),*}) => {
        #[derive(Clone)]
        #[doc = "Boxed type container"]
        pub struct $type_box {
            inner: std::rc::Rc<dyn $trait>,
        }

        impl std::ops::Deref for $type_box {
            type Target = dyn $trait;

            fn deref(&self) -> &Self::Target {
                &*self.inner
            }
        }

        impl $type_box {
            /// Construct a new type in a type box.
            pub fn new<T>(inner: T) -> Self where T: 'static + $trait {
                Self {
                    inner: std::rc::Rc::new(inner),
                }
            }
        }

        impl std::fmt::Debug for $type_box {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.inner.as_enum().fmt(fmt)
            }
        }

        impl std::cmp::PartialEq for $type_box {
            fn eq(&self, other: &Self) -> bool {
                self.inner.as_enum() == other.inner.as_enum()
            }
        }

        impl std::cmp::Eq for $type_box {}

        impl std::hash::Hash for $type_box {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.as_enum().hash(state);
            }
        }

        impl std::cmp::PartialOrd for $type_box {
            fn partial_cmp(&self, other: &$type_box) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl std::cmp::Ord for $type_box {
            fn cmp(&self, other: &$type_box) -> std::cmp::Ordering {
                self.inner.as_enum().cmp(&other.inner.as_enum())
            }
        }

        #[doc = "Enum that can be used for casting between variants of the same type"]
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $enum<$lt> {
            $(
                #[doc = "Type variant"]
                $ty(&$lt $ty),
            )*
        }

        $(
            impl From<$ty> for $type_box {
                fn from(value: $ty) -> $type_box {
                    $type_box::new(value)
                }
            }

            impl_lang_item!($ty, $lang);
        )*

        impl_variadic_type_args!($args, $trait, $type_box);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Ext as _, JavaScript, Tokens};

    #[test]
    fn test_quoted() {
        let n1: Tokens<JavaScript> = toks!("var v = ", "bar".quoted(), ";");
        assert_eq!("var v = \"bar\";", n1.to_string().unwrap().as_str());
    }

    #[test]
    fn test_macros() {
        let mut t = Tokens::<JavaScript>::new();

        push!(t, |t| {
            push!(t, "function bar(a, b) {");
            nested!(t, |t| {
                push!(t, "var v = a + b;");
                push!(t, "return v;");
            });
            push!(t, "}");
        });
        push!(t, "var foo = bar();");

        let expected = vec![
            "function bar(a, b) {",
            "    var v = a + b;",
            "    return v;",
            "}",
            "var foo = bar();",
            "",
        ];

        assert_eq!(expected, t.to_file_vec().unwrap());
    }
}
