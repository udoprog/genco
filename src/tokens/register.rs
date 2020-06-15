use crate::lang::Lang;
use crate::tokens::{FormatInto, RegisterTokens};
use crate::Tokens;

/// Function to provide item registration.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
///
/// # fn main() -> genco::fmt::Result {
/// let write_bytes_ext = rust::import("byteorder", "WriteBytesExt").with_alias("_");
/// let read_bytes_ext = rust::import("byteorder", "ReadBytesExt").with_alias("_");
/// let cursor = &rust::import("std::io", "Cursor");
/// let big_endian = &rust::import("byteorder", "BigEndian");
///
/// let tokens = quote! {
///     #(register((write_bytes_ext, read_bytes_ext)))
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
pub fn register<T, L>(inner: T) -> Register<T>
where
    T: RegisterTokens<L>,
    L: Lang,
{
    Register { inner }
}

/// Struct containing a type only intended to be registered.
///
/// This is constructed with the [register()] function.
#[derive(Clone, Copy)]
pub struct Register<T> {
    inner: T,
}

impl<T, L> FormatInto<L> for Register<T>
where
    L: Lang,
    T: RegisterTokens<L>,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        self.inner.register_tokens(tokens);
    }
}
