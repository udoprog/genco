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
                crate::LangBox::from(Rc::new(value) as Rc<dyn crate::LangItem<$from_lang>>)
            }
        }

        impl<'a> From<&'a $box_from_ty> for crate::LangBox<$from_lang> {
            fn from(value: &'a $box_from_ty) -> Self {
                use std::rc::Rc;
                crate::LangBox::from(Rc::new(value.clone()) as Rc<dyn crate::LangItem<$from_lang>>)
            }
        }
        )?

        $(
            impl crate::LangItem<$lang> for $ty {
                $($item)*

                fn eq(&self, other: &dyn crate::LangItem<$lang>) -> bool {
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
    ($args_vis:vis $args:ident, $type_trait:ident, $any_type:ident) => {
        /// Helper trait for things that can be turned into generic arguments.
        $args_vis trait $args {
            /// Convert the given type into a collection of arguments.
            fn into_args(self) -> Vec<$any_type>;
        }

        impl<T> $args for T
        where
            T: Into<$any_type>,
        {
            fn into_args(self) -> Vec<$any_type> {
                vec![self.into()]
            }
        }

        impl_variadic_type_args!(@args $args, $type_trait, $any_type, A => a);
        impl_variadic_type_args!(@args $args, $type_trait, $any_type, A => a, B => b);
        impl_variadic_type_args!(@args $args, $type_trait, $any_type, A => a, B => b, C => c);
        impl_variadic_type_args!(@args $args, $type_trait, $any_type, A => a, B => b, C => c, D => d);
        impl_variadic_type_args!(@args $args, $type_trait, $any_type, A => a, B => b, C => c, D => d, E => e);
    };

    (@args $args:ty, $ty:ident, $any_type:ident, $($ident:ident => $var:ident),*) => {
        impl<$($ident,)*> $args for ($($ident,)*) where $($ident: Into<$any_type>,)* {
            fn into_args(self) -> Vec<$any_type> {
                let ($($var,)*) = self;
                vec![$($var.into(),)*]
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
        $type_trait_vis:vis trait $type_trait:ident {
            $($type_trait_item:tt)*
        }

        $args_vis:vis trait $args:ident;
        $any_type_vis:vis struct $any_type:ident;
        $any_type_ref_vis:vis enum $any_type_ref:ident;

        $(impl $trait_impl:ident for $ty:ident {
            $($ty_item:tt)*
        })*
    ) => {
        /// Trait implemented by all types
        $type_trait_vis trait TypeTrait: 'static + fmt::Debug + crate::LangItem<$lang> {
            /// Coerce trait into an enum that can be used for type-specific operations
            fn as_enum(&self) -> $any_type_ref<'_>;

            $($type_trait_item)*
        }

        /// Private internals for the any type.
        #[derive(Clone)]
        enum AnyInner {
            $(
                #[doc = "Generated variant."]
                $ty(std::rc::Rc<$ty>),
            )*
        }

        #[derive(Clone)]
        #[doc = "Type that can contain any language type. Derefs to the type trait."]
        $any_type_vis struct $any_type {
            inner: AnyInner,
        }

        $(
            impl From<$ty> for $any_type {
                fn from(value: $ty) -> Self {
                    Self {
                        inner: AnyInner::$ty(std::rc::Rc::new(value))
                    }
                }
            }
        )*

        impl crate::FormatTokens<$lang> for $any_type {
            fn format_tokens(self, tokens: &mut $crate::Tokens<$lang>) {
                let value = match self.inner {
                    $(AnyInner::$ty(value) => value as std::rc::Rc<dyn LangItem<$lang>>,)*
                };

                tokens.item(crate::Item::LangBox(value.into()));
            }
        }

        impl<'a> crate::FormatTokens<$lang> for &'a $any_type {
            fn format_tokens(self, tokens: &mut $crate::Tokens<$lang>) {
                let value = match &self.inner {
                    $(AnyInner::$ty(value) => value.clone() as std::rc::Rc<dyn LangItem<$lang>>,)*
                };

                tokens.item(crate::Item::LangBox(value.into()));
            }
        }

        impl std::ops::Deref for $any_type {
            type Target = dyn $type_trait;

            fn deref(&self) -> &Self::Target {
                match &self.inner {
                    $(AnyInner::$ty(value) => &**value,)*
                }
            }
        }

        impl std::fmt::Debug for $any_type {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.as_enum().fmt(fmt)
            }
        }

        impl std::cmp::PartialEq for $any_type {
            fn eq(&self, other: &Self) -> bool {
                self.as_enum() == other.as_enum()
            }
        }

        impl std::cmp::Eq for $any_type {}

        impl std::hash::Hash for $any_type {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.as_enum().hash(state);
            }
        }

        impl std::cmp::PartialOrd for $any_type {
            fn partial_cmp(&self, other: &$any_type) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl std::cmp::Ord for $any_type {
            fn cmp(&self, other: &$any_type) -> std::cmp::Ordering {
                self.as_enum().cmp(&other.as_enum())
            }
        }

        #[doc = "Enum that can be used for casting between variants of the same type"]
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        $any_type_ref_vis enum $any_type_ref<'a> {
            $(
                #[doc = "Type variant"]
                $ty(&'a $ty),
            )*
        }

        $(
            impl $trait_impl for $ty {
                fn as_enum(&self) -> $any_type_ref<'_> {
                    $any_type_ref::$ty(self)
                }

                $($ty_item)*
            }

            impl_lang_item! {
                impl FormatTokens<$lang> for $ty;
                impl From<$ty> for LangBox<$lang>;
            }
        )*

        impl_variadic_type_args!($args_vis $args, $type_trait, $any_type);
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
