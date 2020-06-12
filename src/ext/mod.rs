//! Extension traits for working with genco.

use crate::lang::Lang;
use crate::tokens::{ItemStr, RegisterTokens};
use std::fmt;

mod display;
mod quoted;
mod register;

pub use self::display::Display;
pub use self::quoted::Quoted;
pub use self::register::Register;

/// Extension traits for language-specific quoting.
pub trait QuotedExt {
    /// Trait to provide string quoting through `<stmt>.quoted()`.
    ///
    /// Note that quoting is applied automatically for literal strings inside of
    /// the [quote!] macro, like: `quote!("hello")`.
    ///
    /// This is used to generated quoted strings, in the language of choice.
    ///
    /// # Examples
    ///
    /// Example showcasing quoted strings when generating Rust.
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let map = rust::imported("std::collections", "HashMap");
    ///
    /// let tokens = quote! {
    ///     let mut m = #map::<u32, &str>::new();
    ///     m.insert(0, #("hello\" world".quoted()));
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///        "use std::collections::HashMap;",
    ///        "",
    ///        "let mut m = HashMap::<u32, &str>::new();",
    ///        "m.insert(0, \"hello\\\" world\");",
    ///     ],
    ///     tokens.to_file_vec()?,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [quote!]: macro.quote.html
    fn quoted(self) -> Quoted<Self>
    where
        Self: Into<ItemStr>,
    {
        Quoted::new(self)
    }
}

impl<T> QuotedExt for T where T: Into<ItemStr> {}

/// Tokenizer for anything that implements display.
pub trait DisplayExt {
    /// Trait to build a string literal through `<stmt>.display()`.
    ///
    /// This is an alternative to manually implementing [tokens::FormatInto], since
    /// it can tokenize anything that implements [Display][fmt::Display]
    /// directly.
    ///
    /// On the other hand, things implementing [tokens::FormatInto] have access to the
    /// full range of the [Tokens] api, allowing it to work more efficiently.
    ///
    /// [tokens::FormatInto]: crate::tokens::FormatInto
    /// [Tokens]: crate::Tokens
    ///
    /// # Examples
    ///
    /// Example showcasing quoted strings when generating Rust.
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use std::fmt;
    ///
    /// # fn main() -> genco::fmt::Result {
    /// struct Foo(());
    ///
    /// impl fmt::Display for Foo {
    ///     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    ///         write!(fmt, "Foo")
    ///     }
    /// }
    ///
    /// let map = rust::imported("std::collections", "HashMap");
    ///
    /// let foo = Foo(());
    ///
    /// let tokens = quote! {
    ///     let mut m = #map::<u32, &str>::new();
    ///     m.insert(0, #(foo.display()));
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///        "use std::collections::HashMap;",
    ///        "",
    ///        "let mut m = HashMap::<u32, &str>::new();",
    ///        "m.insert(0, Foo);",
    ///     ],
    ///     tokens.to_file_vec()?,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn display(&self) -> Display<'_, Self>
    where
        Self: Sized + fmt::Display,
    {
        Display::new(self)
    }
}

impl<T> DisplayExt for T where T: fmt::Display {}

/// Extension traits for interacting with registering.
pub trait RegisterExt<L> {
    /// Trait to provide item registration through `<stmt>.register()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    /// use rust::{imported, Config};
    ///
    /// # fn main() -> genco::fmt::Result {
    /// let write_bytes_ext = imported("byteorder", "WriteBytesExt").alias("_");
    /// let read_bytes_ext = imported("byteorder", "ReadBytesExt").alias("_");
    /// let cursor = &imported("std::io", "Cursor");
    /// let big_endian = &imported("byteorder", "BigEndian");
    ///
    /// let tokens = quote! {
    ///     #((write_bytes_ext, read_bytes_ext).register())
    ///
    ///     let mut wtr = vec![];
    ///     wtr.write_u16::<#big_endian>(517).unwrap();
    ///     wtr.write_u16::<#big_endian>(768).unwrap();
    ///     assert_eq!(wtr, vec![2, 5, 3, 0]);
    ///
    ///     let mut rdr = #cursor::new(vec![2, 5, 3, 0]);
    ///     assert_eq!(517, rdr.read_u16::<#big_endian>().unwrap());
    ///     assert_eq!(768, rdr.read_u16::<#big_endian>().unwrap());
    /// };
    ///
    /// assert_eq!(
    ///     vec![
    ///         "use byteorder::{BigEndian, ReadBytesExt as _, WriteBytesExt as _};",
    ///         "use std::io::Cursor;",
    ///         "",
    ///         "let mut wtr = vec![];",
    ///         "wtr.write_u16::<BigEndian>(517).unwrap();",
    ///         "wtr.write_u16::<BigEndian>(768).unwrap();",
    ///         "assert_eq!(wtr, vec![2, 5, 3, 0]);",
    ///         "",
    ///         "let mut rdr = Cursor::new(vec![2, 5, 3, 0]);",
    ///         "assert_eq!(517, rdr.read_u16::<BigEndian>().unwrap());",
    ///         "assert_eq!(768, rdr.read_u16::<BigEndian>().unwrap());"
    ///     ],
    ///     tokens.to_file_vec()?,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn register(self) -> Register<Self>
    where
        Self: Sized + RegisterTokens<L>,
        L: Lang,
    {
        Register::new(self)
    }
}

impl<T, L> RegisterExt<L> for T
where
    T: RegisterTokens<L>,
    L: Lang,
{
}
