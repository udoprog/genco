//! Macros helpers in genco.

/// Macro to implement support for a custom language.
///
/// # Examples
///
/// ```
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
///         type Item = Any;
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
///                 match import.kind() {
///                     AnyKind::Import(import) => {
///                         header.push();
///                         quote_in!(header => import $(import.0));
///                     }
///                     AnyKind::ImportDefault(import) => {
///                         header.push();
///                         quote_in!(header => import default $(import.0));
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
///     Import(Import) {
///         fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, _: &Format) -> fmt::Result {
///             out.write_str(self.0)?;
///             Ok(())
///         }
///     }
///
///     ImportDefault(ImportDefault) {
///         fn format(&self, out: &mut fmt::Formatter<'_>, config: &Config, _: &Format) -> fmt::Result {
///             write!(out, "default:{}", self.0)?;
///             Ok(())
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
/// let a = Import("first");
/// let b = ImportDefault("second");
///
/// let t: Tokens<MyLang> = quote! {
///     $a
///     $b
/// };
///
/// assert_eq! {
///     vec![
///         "import default second",
///         "import first",
///         "",
///         "first",
///         "default:second"
///     ],
///     t.to_file_vec()?
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[macro_export]
macro_rules! impl_lang {
    (
        $(#[$($meta:meta)*])*
        $vis:vis $lang:ident {
            $($lang_item:tt)*
        }

        $(
            $name:ident($ty:ty) {
                $($ty_lang_item_item:tt)*
            }
        )*
    ) => {
        $(#[$($meta)*])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $lang(());

        impl $crate::lang::Lang for $lang {
            $($lang_item)*
        }

        #[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        enum AnyKind {
            $($name($ty),)*
        }

        /// A type-erased language item capable of holding any kind.
        #[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        $vis struct Any {
            kind: AnyKind,
        }

        impl Any {
            /// Access the kind of the any type.
            #[allow(unused)]
            fn kind(&self) -> &AnyKind {
                &self.kind
            }
        }

        $(
            impl From<$ty> for Any {
                #[inline]
                fn from(lang: $ty) -> Self {
                    Any { kind: AnyKind::$name(lang) }
                }
            }
        )*

        impl $crate::lang::LangItem<$lang> for Any {
            #[inline]
            fn format(
                &self,
                out: &mut $crate::fmt::Formatter<'_>,
                config: &<$lang as $crate::lang::Lang>::Config,
                format: &<$lang as $crate::lang::Lang>::Format,
            ) -> $crate::fmt::Result {
                match &self.kind {
                    $(AnyKind::$name(lang) => lang.format(out, config, format),)*
                }
            }
        }

        $(
            impl $crate::tokens::FormatInto<$lang> for $ty {
                fn format_into(self, tokens: &mut $crate::Tokens<$lang>) {
                    tokens.append($crate::__priv::item::<$lang>(self.into()));
                }
            }

            impl<'a> $crate::tokens::FormatInto<$lang> for &'a $ty {
                fn format_into(self, tokens: &mut $crate::Tokens<$lang>) {
                    tokens.append($crate::__priv::item::<$lang>(self.clone().into()));
                }
            }

            impl $crate::tokens::Register<$lang> for $ty {
                fn register(self, tokens: &mut $crate::Tokens<$lang>) {
                    tokens.append($crate::__priv::register::<$lang>(self.into()));
                }
            }

            impl<'a> $crate::tokens::Register<$lang> for &'a $ty {
                fn register(self, tokens: &mut $crate::Tokens<$lang>) {
                    tokens.append($crate::__priv::register::<$lang>(self.clone().into()));
                }
            }

            impl $crate::lang::LangItem<$lang> for $ty {
                $($ty_lang_item_item)*
            }
        )*
    }
}
