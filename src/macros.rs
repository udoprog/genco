//! Macros helpers in genco.

macro_rules! impl_lang_item {
    (
        $(impl FormatTokens<$from_lang:ty> for $from_ty:ident;)?
        $(impl From<$box_from_ty:ident> for LangBox<$box_lang:ty>;)?

        $(impl LangItem<$lang:ty> for $ty:ident {
            $($item:item)*
        })?
    ) => {
        $(
        impl crate::FormatTokens<$from_lang> for $from_ty {
            fn format_tokens(self, tokens: &mut crate::Tokens<$from_lang>) {
                tokens.item(crate::Item::LangBox(self.into()));
            }
        }

        impl<'a> crate::FormatTokens<$from_lang> for &'a $from_ty {
            fn format_tokens(self, tokens: &mut crate::Tokens<$from_lang>) {
                tokens.item(crate::Item::LangBox(self.into()));
            }
        }
        )?

        $(
        impl From<$box_from_ty> for crate::LangBox<$from_lang> {
            fn from(value: $box_from_ty) -> Self {
                use std::rc::Rc;
                crate::LangBox::from(Rc::new(value) as Rc<dyn LangItem<$from_lang>>)
            }
        }

        impl<'a> From<&'a $box_from_ty> for crate::LangBox<$from_lang> {
            fn from(value: &'a $box_from_ty) -> Self {
                use std::rc::Rc;
                crate::LangBox::from(Rc::new(value.clone()) as Rc<dyn LangItem<$from_lang>>)
            }
        }
        )?

        $(
            impl LangItem<$lang> for $ty {
                $($item)*

                fn eq(&self, other: &dyn LangItem<$lang>) -> bool {
                    other
                        .as_any()
                        .downcast_ref::<Self>()
                        .map_or(false, |x| x == self)
                }

                fn as_any(&self) -> &dyn std::any::Any {
                    self
                }
            }
            )?
    }
}

macro_rules! impl_variadic_type_args {
    ($args_vis:vis $args:ident, $trait:ident, $type_box:ident) => {
        /// Helper trait for things that can be turned into generic arguments.
        $args_vis trait $args {
            /// Convert the given type into a collection of arguments.
            fn into_args(self) -> Vec<$type_box>;
        }

        impl<T> $args for T
        where
            T: 'static + $trait,
        {
            fn into_args(self) -> Vec<$type_box> {
                vec![$type_box::new(self)]
            }
        }

        impl_variadic_type_args!(@args $args, $trait, $type_box, A => a);
        impl_variadic_type_args!(@args $args, $trait, $type_box, A => a, B => b);
        impl_variadic_type_args!(@args $args, $trait, $type_box, A => a, B => b, C => c);
        impl_variadic_type_args!(@args $args, $trait, $type_box, A => a, B => b, C => c, D => d);
        impl_variadic_type_args!(@args $args, $trait, $type_box, A => a, B => b, C => c, D => d, E => e);
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

macro_rules! impl_dynamic_types {
    ($lang:ty =>
        $trait_vis:vis trait $trait:ident {
            $($trait_item:tt)*
        }

        $args_vis:vis trait $args:ident;
        $type_box_vis:vis struct $type_box:ident;
        $enum_vis:vis enum $enum:ident;

        $(impl $trait_impl:ident for $ty:ident {
            $($ty_item:tt)*
        })*
    ) => {
        /// Trait implemented by all types
        $trait_vis trait TypeTrait: 'static + fmt::Debug + LangItem<$lang> {
            /// Coerce trait into an enum that can be used for type-specific operations
            fn as_enum(&self) -> $enum<'_>;

            $($trait_item)*
        }

        #[derive(Clone)]
        #[doc = "Boxed type container"]
        $type_box_vis struct $type_box {
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
        $enum_vis enum $enum<'a> {
            $(
                #[doc = "Type variant"]
                $ty(&'a $ty),
            )*
        }

        $(
            impl $trait_impl for $ty {
                fn as_enum(&self) -> $enum<'_> {
                    $enum::$ty(self)
                }

                $($ty_item)*
            }

            impl From<$ty> for $type_box {
                fn from(value: $ty) -> $type_box {
                    $type_box::new(value)
                }
            }

            impl_lang_item! {
                impl FormatTokens<$lang> for $ty;
                impl From<$ty> for LangBox<$lang>;
            }
        )*

        impl_variadic_type_args!($args_vis $args, $trait, $type_box);
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
                    tokens.space();
                    modifier.format_tokens(tokens);
                }
            }
        }
    }
}
