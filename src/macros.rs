//! Macros helpers in genco.

macro_rules! impl_dynamic_types {
    ($(#[$($meta:meta)*])* $vis:vis $lang:ident =>
        $(
            $ty:ident {
                impl LangItem {
                    $($ty_lang_item_item:tt)*
                }
            }
        )*
    ) => {
        $(#[$($meta)*])*
        $vis struct $lang(());

        /// Trait implemented by all language items.
        $vis trait AsAny: crate::lang::LangItem<$lang> {
            /// Coerce trait into an enum that can be used for type-specific operations.
            fn as_any(&self) -> Any<'_>;
        }

        #[doc = "Enum that can be used for casting between variants of the same type"]
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        $vis enum Any<'a> {
            $(
                #[doc = "Type variant"]
                $ty(&'a $ty),
            )*
        }

        $(
            impl AsAny for $ty {
                fn as_any(&self) -> Any<'_> {
                    Any::$ty(self)
                }
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

            impl crate::lang::LangItem<$lang> for $ty {
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
    }
}
