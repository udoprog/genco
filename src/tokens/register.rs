use crate::lang::Lang;
use crate::tokens::FormatInto;
use crate::Tokens;

/// Function to provide item registration.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let write_bytes_ext = rust::import("byteorder", "WriteBytesExt").with_alias("_");
/// let read_bytes_ext = rust::import("byteorder", "ReadBytesExt").with_alias("_");
/// let cursor = &rust::import("std::io", "Cursor");
/// let big_endian = &rust::import("byteorder", "BigEndian");
///
/// let tokens = quote! {
///     $(register((write_bytes_ext, read_bytes_ext)))
///
///     let mut wtr = vec![];
///     wtr.write_u16::<$big_endian>(517).unwrap();
///     wtr.write_u16::<$big_endian>(768).unwrap();
///     assert_eq!(wtr, vec![2, 5, 3, 0]);
///
///     let mut rdr = $cursor::new(vec![2, 5, 3, 0]);
///     assert_eq!(517, rdr.read_u16::<$big_endian>().unwrap());
///     assert_eq!(768, rdr.read_u16::<$big_endian>().unwrap());
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
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn register<T, L>(inner: T) -> RegisterFn<T>
where
    T: Register<L>,
    L: Lang,
{
    RegisterFn { inner }
}

/// Struct containing a type only intended to be registered.
///
/// This is constructed with the [register()] function.
#[derive(Debug, Clone, Copy)]
pub struct RegisterFn<T> {
    inner: T,
}

impl<T, L> FormatInto<L> for RegisterFn<T>
where
    T: Register<L>,
    L: Lang,
{
    fn format_into(self, t: &mut Tokens<L>) {
        t.register(self.inner);
    }
}

/// Helper trait to convert something into a stream of registrations.
///
/// Thanks to this, we can provide a flexible number of arguments to
/// [register()], like a tuple.
///
/// This is trait is very similar to [FormatInto], except that it constrains
/// the types that can be "registered" to only language items. An implementation
/// of register must not be used as a substitute for [FormatInto].
///
/// [register()]: crate::Tokens::register()
/// [FormatInto]: crate::tokens::FormatInto
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let mut tokens = rust::Tokens::new();
///
/// let hash_map = rust::import("std::collections", "HashMap");
/// let btree_map = rust::import("std::collections", "BTreeMap");
///
/// tokens.register((hash_map, btree_map));
///
/// assert_eq!(
///     vec![
///         "use std::collections::{BTreeMap, HashMap};",
///     ],
///     tokens.to_file_vec()?,
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub trait Register<L>
where
    L: Lang,
{
    /// Convert the type into tokens.
    fn register(self, tokens: &mut Tokens<L>);
}

/// Macro to build implementations of `Register<T>` for a tuple.
macro_rules! impl_register_tuple {
    ($($ty:ident, $var:ident),*) => {
        impl<L, $($ty,)*> Register<L> for ($($ty,)*)
        where
            $($ty: Register<L>,)*
            L: Lang,
        {
            fn register(self, tokens: &mut Tokens<L>) {
                let ($($var,)*) = self;
                $(tokens.register($var);)*
            }
        }
    }
}

impl_register_tuple!(T1, t1);
impl_register_tuple!(T1, t1, T2, t2);
impl_register_tuple!(T1, t1, T2, t2, T3, t3);
impl_register_tuple!(T1, t1, T2, t2, T3, t3, T4, t4);
impl_register_tuple!(T1, t1, T2, t2, T3, t3, T4, t4, T5, t5);
impl_register_tuple!(T1, t1, T2, t2, T3, t3, T4, t4, T5, t5, T6, t6);
impl_register_tuple!(T1, t1, T2, t2, T3, t3, T4, t4, T5, t5, T6, t6, T7, t7);
impl_register_tuple!(T1, t1, T2, t2, T3, t3, T4, t4, T5, t5, T6, t6, T7, t7, T8, t8);
