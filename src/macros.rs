//! Macros helpers in genco.

/// Macro to implement support for a custom language.
///
/// # Examples
///
/// ```rust
/// use genco::fmt;
/// use std::fmt::Write as _;
///
/// #[derive(Default)]
/// struct Config {
/// }
///
/// #[derive(Default)]
/// struct Format {
/// }
///
/// genco::impl_lang! {
///     MyLang {
///         type Config = Config;
///         type Import = dyn AsAny;
///         type Format = Format;
///
///         fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
///             genco::lang::c_family_write_quoted(out, input)
///         }
///
///         fn format_file(
///             tokens: &Tokens<MyLang>,
///             out: &mut fmt::Formatter<'_>,
///             config: &Self::Config,
///         ) -> fmt::Result {
///             use genco::quote_in;
///
///             let mut header: Tokens<MyLang> = Tokens::new();
///             let mut any_imports = false;
///
///             for import in tokens.walk_imports() {
///                 any_imports = true;
///
///                 match import.as_any() {
///                     Any::Import(import) => {
///                         header.push();
///                         quote_in!(header => import #(import.0));
///                     }
///                     Any::ImportDefault(import) => {
///                         header.push();
///                         quote_in!(header => import default #(import.0));
///                     }
///                 }
///             }
///
///             if any_imports {
///                 // Add a line as padding in case we have any imports.
///                 header.line();
///             }
///
///             let format = Format::default();
///             header.format(out, config, &format)?;
///             tokens.format(out, config, &format)?;
///             Ok(())
///         }
///     }
///
///     Import {
///         fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, _: &Format) -> fmt::Result {
///             out.write_str(self.0)?;
///             Ok(())
///         }
///
///         fn as_import(&self) -> Option<&dyn AsAny> {
///             Some(self)
///         }
///     }
///
///     ImportDefault {
///         fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, _: &Format) -> fmt::Result {
///             write!(out, "default:{}", self.0)?;
///             Ok(())
///         }
///
///         fn as_import(&self) -> Option<&dyn AsAny> {
///             Some(self)
///         }
///     }
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// struct Import(&'static str);
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// struct ImportDefault(&'static str);
///
/// use genco::{quote, Tokens};
///
/// # fn main() -> genco::fmt::Result {
/// let a = Import("first");
/// let b = ImportDefault("second");
///
/// let t: Tokens<MyLang> = quote! {
///     #a
///     #b
/// };
///
/// assert_eq! {
///     vec![
///         "import first",
///         "import default second",
///         "",
///         "first",
///         "default:second"
///     ],
///     t.to_file_vec()?
/// };
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! impl_lang {
    (
        $(#[$($meta:meta)*])*
        $vis:vis $lang:ident {
            $($lang_item:tt)*
        }

        $(
            $ty:ident {
                $($ty_lang_item_item:tt)*
            }
        )*
    ) => {
        $(#[$($meta)*])*
        $vis struct $lang(());

        impl $crate::lang::Lang for $lang {
            $($lang_item)*
        }

        /// Language-specific conversion trait implemented by all language
        /// items.
        $vis trait AsAny: $crate::lang::LangItem<$lang> {
            /// Coerce trait into an enum that can be used for type-specific
            /// operations.
            ///
            /// # Examples
            ///
            /// ```rust
            /// genco::impl_lang! {
            ///     MyLang {
            ///         type Config = ();
            ///         type Import = dyn AsAny;
            ///         type Format = ();
            ///     }
            ///
            ///     Import {
            ///         fn as_import(&self) -> Option<&dyn AsAny> {
            ///             Some(self)
            ///         }
            ///     }
            /// };
            ///
            /// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            /// struct Import(usize);
            ///
            /// use genco::{quote, Tokens};
            ///
            /// let t: Tokens<MyLang> = quote! {
            ///     #(Import(0))
            ///     #(Import(1))
            /// };
            ///
            /// /// Find and compare all imports.
            /// assert_eq!(2, t.walk_imports().count());
            ///
            /// for (i, import) in t.walk_imports().enumerate() {
            ///     assert_eq!(Any::Import(&Import(i)), import.as_any());
            /// }
            /// ```
            fn as_any(&self) -> Any<'_>;
        }

        /// Enum produced by [AsAny::as_any()] which can be used to identify and
        /// operate over a discrete language item type.
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        $vis enum Any<'a> {
            $(
                #[doc = "Type variant."]
                $ty(&'a $ty),
            )*
        }

        $(
            impl AsAny for $ty {
                fn as_any(&self) -> Any<'_> {
                    Any::$ty(self)
                }
            }

            impl $crate::tokens::FormatInto<$lang> for $ty {
                fn format_into(self, tokens: &mut $crate::Tokens<$lang>) {
                    tokens.append($crate::tokens::Item::Lang(Box::new(self)));
                }
            }

            impl<'a> $crate::tokens::FormatInto<$lang> for &'a $ty {
                fn format_into(self, tokens: &mut $crate::Tokens<$lang>) {
                    tokens.append($crate::tokens::Item::Lang(Box::new(self.clone())));
                }
            }

            impl $crate::tokens::Register<$lang> for $ty {
                fn register(self, tokens: &mut $crate::Tokens<$lang>) {
                    tokens.append($crate::tokens::Item::Register(Box::new(self)));
                }
            }

            impl<'a> $crate::tokens::Register<$lang> for &'a $ty {
                fn register(self, tokens: &mut $crate::Tokens<$lang>) {
                    tokens.append($crate::tokens::Item::Register(Box::new(self.clone())));
                }
            }

            impl $crate::lang::LangItem<$lang> for $ty {
                $($ty_lang_item_item)*

                fn __lang_item_as_any(&self) -> &dyn std::any::Any {
                    self
                }

                fn __lang_item_clone(&self) -> Box<dyn $crate::lang::LangItem<$lang>> {
                    Box::new(self.clone())
                }

                fn __lang_item_eq(&self, other: &dyn $crate::lang::LangItem<$lang>) -> bool {
                    other
                        .__lang_item_as_any()
                        .downcast_ref::<Self>()
                        .map_or(false, |x| x == self)
                }
            }
        )*
    }
}
