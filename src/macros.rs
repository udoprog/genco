//! Macros helpers in genco.

macro_rules! impl_lang_item {
    (
        impl LangItem<$lang:ty> for $ty:ident {
            $($item:item)*
        }
    ) => {
        impl crate::tokens::FormatInto<$lang> for $ty {
            fn format_into(self, tokens: &mut crate::Tokens<$lang>) {
                tokens.item(crate::tokens::Item::LangBox(self.into()));
            }
        }

        impl<'a> crate::tokens::FormatInto<$lang> for &'a $ty {
            fn format_into(self, tokens: &mut crate::Tokens<$lang>) {
                tokens.item(crate::tokens::Item::LangBox(self.into()));
            }
        }

        impl From<$ty> for crate::lang::LangBox<$lang> {
            fn from(value: $ty) -> Self {
                crate::lang::LangBox::from(Box::new(value) as Box<dyn crate::lang::LangItem<$lang>>)
            }
        }

        impl<'a> From<&'a $ty> for crate::lang::LangBox<$lang> {
            fn from(value: &'a $ty) -> Self {
                crate::lang::LangBox::from(Box::new(value.clone()) as Box<dyn crate::lang::LangItem<$lang>>)
            }
        }

        impl crate::lang::LangItem<$lang> for $ty {
            $($item)*

            fn __lang_item_as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn __lang_item_clone(&self) -> Box<dyn crate::lang::LangItem<$lang>> {
                Box::new(self.clone())
            }

            fn __lang_item_eq(&self, other: &dyn crate::lang::LangItem<$lang>) -> bool {
                other
                    .__lang_item_as_any()
                    .downcast_ref::<Self>()
                    .map_or(false, |x| x == self)
            }
        }
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
        trait TypeTrait {
            $($type_trait_item:tt)*
        }

        $(
            $ty:ident {
                impl TypeTrait {
                    $($ty_item:tt)*
                }

                impl LangItem {
                    $($ty_lang_item_item:tt)*
                }
            }
        )*
    ) => {
        /// Trait implemented by all types
        pub trait TypeTrait: 'static + std::fmt::Debug + crate::lang::LangItem<$lang> {
            /// Coerce trait into an enum that can be used for type-specific operations
            fn as_enum(&self) -> AnyRef<'_>;

            $($type_trait_item)*
        }

        /// Private internals for the any type.
        #[derive(Clone)]
        enum AnyInner {
            $(
                #[doc = "Generated variant."]
                $ty(Box<$ty>),
            )*
        }

        #[derive(Clone)]
        #[doc = "Type that can contain any language type. Derefs to the type trait."]
        pub struct Any {
            inner: AnyInner,
        }

        $(
            impl From<$ty> for Any {
                fn from(value: $ty) -> Self {
                    Self {
                        inner: AnyInner::$ty(Box::new(value))
                    }
                }
            }
        )*

        impl crate::tokens::FormatInto<$lang> for Any {
            fn format_into(self, tokens: &mut $crate::Tokens<$lang>) {
                let value = match self.inner {
                    $(AnyInner::$ty(value) => value as Box<dyn LangItem<$lang>>,)*
                };

                tokens.item(crate::tokens::Item::LangBox(value.into()));
            }
        }

        impl<'a> crate::tokens::FormatInto<$lang> for &'a Any {
            fn format_into(self, tokens: &mut $crate::Tokens<$lang>) {
                let value = match &self.inner {
                    $(AnyInner::$ty(value) => Box::new(Clone::clone(&**value)) as Box<dyn LangItem<$lang>>,)*
                };

                tokens.item(crate::tokens::Item::LangBox(value.into()));
            }
        }

        impl std::ops::Deref for Any {
            type Target = dyn TypeTrait;

            fn deref(&self) -> &Self::Target {
                match &self.inner {
                    $(AnyInner::$ty(value) => &**value,)*
                }
            }
        }

        impl std::fmt::Debug for Any {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.as_enum().fmt(fmt)
            }
        }

        impl std::cmp::PartialEq for Any {
            fn eq(&self, other: &Self) -> bool {
                self.as_enum() == other.as_enum()
            }
        }

        impl std::cmp::Eq for Any {}

        impl std::hash::Hash for Any {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.as_enum().hash(state);
            }
        }

        impl std::cmp::PartialOrd for Any {
            fn partial_cmp(&self, other: &Any) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl std::cmp::Ord for Any {
            fn cmp(&self, other: &Any) -> std::cmp::Ordering {
                self.as_enum().cmp(&other.as_enum())
            }
        }

        #[doc = "Enum that can be used for casting between variants of the same type"]
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub enum AnyRef<'a> {
            $(
                #[doc = "Type variant"]
                $ty(&'a $ty),
            )*
        }

        $(
            impl TypeTrait for $ty {
                fn as_enum(&self) -> AnyRef<'_> {
                    AnyRef::$ty(self)
                }

                $($ty_item)*
            }

            impl crate::tokens::FormatInto<$lang> for $ty {
                fn format_into(self, tokens: &mut crate::Tokens<$lang>) {
                    tokens.item(crate::tokens::Item::LangBox(self.into()));
                }
            }

            impl<'a> crate::tokens::FormatInto<$lang> for &'a $ty {
                fn format_into(self, tokens: &mut crate::Tokens<$lang>) {
                    tokens.item(crate::tokens::Item::LangBox(self.into()));
                }
            }

            impl From<$ty> for crate::lang::LangBox<$lang> {
                fn from(value: $ty) -> Self {
                    crate::lang::LangBox::from(Box::new(value) as Box<dyn crate::lang::LangItem<$lang>>)
                }
            }

            impl<'a> From<&'a $ty> for crate::lang::LangBox<$lang> {
                fn from(value: &'a $ty) -> Self {
                    crate::lang::LangBox::from(Box::new(value.clone()) as Box<dyn crate::lang::LangItem<$lang>>)
                }
            }

            impl LangItem<$lang> for $ty {
                $($ty_lang_item_item)*

                fn __lang_item_as_any(&self) -> &dyn std::any::Any {
                    self
                }

                fn __lang_item_clone(&self) -> Box<dyn crate::lang::LangItem<$lang>> {
                    Box::new(self.clone())
                }

                fn __lang_item_eq(&self, other: &dyn crate::lang::LangItem<$lang>) -> bool {
                    other
                        .__lang_item_as_any()
                        .downcast_ref::<Self>()
                        .map_or(false, |x| x == self)
                }
            }
        )*

        impl_variadic_type_args!(pub Args, TypeTrait, Any);
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

        impl crate::tokens::FormatInto<$lang> for $name {
            fn format_into(self, tokens: &mut crate::Tokens<$lang>) {
                tokens.append(self.name());
            }
        }

        impl crate::tokens::FormatInto<$lang> for Vec<$name> {
            fn format_into(self, tokens: &mut crate::Tokens<$lang>) {
                use std::collections::BTreeSet;

                let mut it = self.into_iter().collect::<BTreeSet<_>>().into_iter();

                if let Some(modifier) = it.next() {
                    modifier.format_into(tokens);
                }

                for modifier in it {
                    tokens.space();
                    modifier.format_into(tokens);
                }
            }
        }
    }
}
