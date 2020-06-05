//! Macros in GenCo

/// Helper macro to reduce boilerplate needed with nested token expressions.
///
/// ## Examples
///
/// ```rust
/// # #![allow(deprecated)]
/// use genco::prelude::*;
/// use genco::toks;
///
/// let n1: Tokens = toks!("var v = ", "bar".quoted(), ";");
/// ```
///
/// ```rust,ignore
/// # #![allow(deprecated)]
/// use genco::prelude::*;
/// use genco::{push, nested};
///
/// let mut t = js::Tokens::new();
///
/// push!(t, |t| {
///     push!(t, "function bar(a, b) {");
///     nested!(t, |t| {
///         push!(t, "var v = a + b;");
///         push!(t, "return v;");
///     });
///     push!(t, "}");
/// });
///
/// push!(t, "var foo = bar();");
///
/// let expected = vec![
///     "function bar(a, b) {",
///     "    var v = a + b;",
///     "    return v;",
///     "}",
///     "var foo = bar();",
///     "",
/// ];
///
/// assert_eq!(expected, t.to_file_vec().unwrap());
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
/// use genco::{Tokens, Java, ItemStr};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = ItemStr::Static("hello");
///
/// push!(toks, "foo ", id);
/// push!(toks, "bar ", id);
///
/// assert_eq!(
///     vec![
///         "foo hello",
///         "bar hello",
///         ""
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// # }
/// ```
///
/// Pushing as a block:
///
/// ```rust
/// # #![allow(deprecated)]
/// # #[macro_use] extern crate genco;
/// # fn main() {
/// use genco::{Tokens, Java, ItemStr};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = ItemStr::from("hello");
///
/// push!(toks, |t| {
///   push!(t, "foo ", id);
///   push!(t, "bar ", id);
/// });
///
/// assert_eq!(
///     vec![
///         "foo hello",
///         "bar hello",
///         ""
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// # }
/// ```
#[macro_export]
#[deprecated(since = "0.5.0", note = "Use the quote! procedural macro instead.")]
macro_rules! push {
    ($dest:expr, |$t:ident| $code:block) => {{
        $dest.append({
            let mut $t = $crate::Tokens::new();
            $code
            $t
        });
        $dest.push();
    }};

    ($dest:expr, $($x:expr),*) => {{
        $dest.append({
            let mut _t = $crate::Tokens::new();
            $(_t.append(Clone::clone(&$x));)*
            _t
        });
        $dest.push();
    }};

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
/// use genco::{Tokens, Java, ItemStr};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = ItemStr::from("hello");
///
/// nested!(toks, "foo ", id);
/// nested!(toks, "bar ", id);
///
/// assert_eq!(
///     vec![
///         "    foo hello",
///         "    bar hello",
///         ""
///     ],
///     toks.to_file_vec().unwrap()
/// );
/// # }
/// ```
///
/// Pushing as a block:
///
/// ```rust
/// # #![allow(deprecated)]
/// # use genco::{nested, push};
/// # fn main() {
/// use genco::{Tokens, Java, ItemStr};
///
/// let mut toks = Tokens::<Java>::new();
/// // id being cloned.
/// let id = ItemStr::from("hello");
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
    ($dest:expr, |$t:ident| $code:block) => {{
        $dest.indent();
        $dest.append({
            let mut $t = $crate::Tokens::new();
            $code
            $t
        });
        $dest.unindent();
    }};

    ($dest:expr, $($x:expr),*) => {
        $dest.indent();
        $dest.append({
            let mut _t = $crate::Tokens::new();
            $(_t.append(Clone::clone(&$x));)*
            _t
        });
        $dest.unindent();
    };

    ($dest:expr, $($x:expr,)*) => {nested!($dest, $($x),*)};
}

macro_rules! impl_lang_item {
    ($ty:ident, $lang:ty) => {
        impl crate::FormatTokens<$lang> for $ty {
            fn format_tokens(self, tokens: &mut crate::Tokens<$lang>) {
                tokens.elements.push(crate::Item::LangBox(self.into()));
            }
        }

        impl<'a> crate::FormatTokens<$lang> for &'a $ty {
            fn format_tokens(self, tokens: &mut crate::Tokens<$lang>) {
                tokens.elements.push(crate::Item::LangBox(self.into()));
            }
        }

        impl From<$ty> for crate::LangBox<$lang> {
            fn from(value: $ty) -> Self {
                use std::rc::Rc;
                crate::LangBox::from(Rc::new(value) as Rc<dyn LangItem<$lang>>)
            }
        }

        impl<'a> From<&'a $ty> for crate::LangBox<$lang> {
            fn from(value: &'a $ty) -> Self {
                use std::rc::Rc;
                crate::LangBox::from(Rc::new(value.clone()) as Rc<dyn LangItem<$lang>>)
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

macro_rules! impl_modifier {
    ($(#[$meta:meta])* pub enum $name:ident<$lang:ty> {
        $(
            $(#[$variant_meta:meta])*
            $variant:ident => $value:expr,
        )*
    }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl $name {
            /// Get the name of the modifier.
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => $value,)*
                }
            }
        }

        impl crate::FormatTokens<$lang> for $name {
            fn format_tokens(self, tokens: &mut crate::Tokens<$lang>) {
                tokens.append(self.name());
            }
        }

        impl crate::FormatTokens<$lang> for Vec<$name> {
            fn format_tokens(self, tokens: &mut crate::Tokens<$lang>) {
                use std::collections::BTreeSet;

                let mut it = self.into_iter().collect::<BTreeSet<_>>().into_iter();

                if let Some(modifier) = it.next() {
                    modifier.format_tokens(tokens);
                }

                for modifier in it {
                    tokens.spacing();
                    modifier.format_tokens(tokens);
                }
            }
        }
    }
}
