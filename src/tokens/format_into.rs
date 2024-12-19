use core::fmt::Arguments;

use alloc::borrow::Cow;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::lang::Lang;
use crate::tokens::{Item, ItemStr, Tokens};

/// Trait for types that can be formatted in-place into a token stream.
///
/// Things implementing [FormatInto] can be used as arguments for
/// [interpolation] in the [quote!] macro.
///
/// [from_fn()] is a helper function which simplifies the task of creating a
/// [FormatInto] implementation on the fly.
///
/// [from_fn()]: crate::tokens::from_fn()
/// [quote!]: macro.quote.html
/// [interpolation]: macro.quote.html#interpolation
///
/// # Examples
///
/// ```
/// use genco::quote_in;
/// use genco::tokens::{ItemStr, FormatInto, from_fn, static_literal};
/// use genco::lang::Lang;
///
/// fn comment<L>(s: impl Into<ItemStr>) -> impl FormatInto<L>
/// where
///     L: Lang
/// {
///     from_fn(move |tokens| {
///         let s = s.into();
///         quote_in!(*tokens => $(static_literal("//")) $s);
///     })
/// }
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub trait FormatInto<L>
where
    L: Lang,
{
    /// Convert the type into tokens in-place.
    ///
    /// A simple way to build ad-hoc format_into implementations is by using
    /// the [from_fn()] function.
    ///
    /// [from_fn()]: crate::tokens::from_fn()
    fn format_into(self, tokens: &mut Tokens<L>);
}

impl<L> FormatInto<L> for Tokens<L>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Self) {
        tokens.extend(self);
    }
}

/// Formatting a reference to a token stream is exactly the same as extending
/// the token stream with a copy of the stream being formatted.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let a: &Tokens = &quote!(foo bar);
///
/// let result = quote!($a baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for &Tokens<L>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.extend(self.iter().cloned());
    }
}

/// Formatting a vector of token streams is like formatting each, one after
/// another.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let mut vec = Vec::<Tokens>::new();
/// vec.push(quote!(foo));
/// vec.push(quote!($[' ']bar));
///
/// let result = quote!($vec baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L, T> FormatInto<L> for Vec<T>
where
    L: Lang,
    T: FormatInto<L>,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        for t in self {
            tokens.append(t);
        }
    }
}

/// Formatting a reference to a vector of token streams is like formatting each,
/// one after another.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let mut vec = Vec::<Tokens>::new();
/// vec.push(quote!(foo));
/// vec.push(quote!($[' ']bar));
///
/// let result = quote!($(&vec) baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L, T> FormatInto<L> for &Vec<T>
where
    L: Lang,
    T: FormatInto<L> + Clone,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        for t in self {
            tokens.append(t.clone());
        }
    }
}

/// Formatting a slice of token streams is like formatting each, one after
/// another.
///
/// This will cause each token stream to be cloned into the destination stream.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let vec = vec!["foo", " ", "bar"];
/// let slice = &vec[..];
///
/// let result: Tokens = quote!($slice baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L, T> FormatInto<L> for &[T]
where
    L: Lang,
    T: Clone + FormatInto<L>,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        for t in self {
            tokens.append(t.clone());
        }
    }
}

/// Formatting borrowed string boxed them on the heap.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let foo = "foo";
/// let bar = "bar";
///
/// let result: Tokens = quote!($foo $bar baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for &str
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(Item::Literal(ItemStr::from(self)));
    }
}

/// Formatting borrowed string boxed them on the heap.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let foo = String::from("foo");
/// let bar = String::from("bar");
///
/// let result: Tokens = quote!($(&foo) $(&bar) baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for &String
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(Item::Literal(ItemStr::from(self)));
    }
}

/// Formatting owned strings takes ownership of the string directly from the
/// heap.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let foo = String::from("foo");
/// let bar = String::from("bar");
///
/// let result: Tokens = quote!($foo $bar baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for String
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(Item::Literal(ItemStr::from(self)));
    }
}

/// Refcounted strings are moved into the token stream without copying.
///
/// # Examples
///
/// ```
/// use std::rc::Rc;
/// use genco::prelude::*;
///
/// let foo = Rc::new(String::from("foo"));
/// let bar = Rc::new(String::from("bar"));
///
/// let result: Tokens = quote!($foo $bar baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for Rc<String>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(Item::Literal(ItemStr::from(self)));
    }
}

/// Refcounted strings are cloned and moved into the token stream without
/// copying.
///
/// # Examples
///
/// ```
/// use std::rc::Rc;
/// use genco::prelude::*;
///
/// let foo = Rc::new(String::from("foo"));
/// let bar = Rc::new(String::from("bar"));
///
/// let result: Tokens = quote!($(&foo) $(&bar) baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for &Rc<String>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(Item::Literal(ItemStr::from(self.clone())));
    }
}

/// Implementation for [Arguments] which allows for arbitrary and efficient
/// literal formatting.
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let name = "John";
/// let result: Tokens = quote!($(format_args!("Hello {name}")));
///
/// assert_eq!("Hello John", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for Arguments<'_>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        tokens.item(Item::Literal(ItemStr::from(self.to_string())));
    }
}

/// Optional items are formatted if they are present.
///
/// # Examples
///
/// ```
/// use std::rc::Rc;
/// use genco::prelude::*;
///
/// let foo = Some("foo");
/// let bar = Some("bar");
/// let biz = None::<&str>;
///
/// let result: Tokens = quote!($foo $bar baz $biz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L, T> FormatInto<L> for Option<T>
where
    L: Lang,
    T: FormatInto<L>,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        if let Some(inner) = self {
            inner.format_into(tokens);
        }
    }
}

/// Cow strings are formatted by either borrowing or cloning the string.
///
/// # Examples
///
/// ```
/// use std::borrow::Cow;
/// use genco::prelude::*;
///
/// let foo = Cow::<str>::Borrowed("foo");
/// let bar = Cow::<str>::Owned(String::from("bar"));
///
/// let result: Tokens = quote!($foo $bar baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for Cow<'_, str>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        match self {
            Cow::Borrowed(b) => tokens.item(Item::Literal(ItemStr::from(b))),
            Cow::Owned(o) => o.format_into(tokens),
        }
    }
}

/// Cow strings are formatted by either borrowing or cloning the string.
///
/// # Examples
///
/// ```
/// use std::borrow::Cow;
/// use genco::prelude::*;
///
/// let foo = &Cow::<str>::Borrowed("foo");
/// let bar = &Cow::<str>::Owned(String::from("bar"));
///
/// let result: Tokens = quote!($foo $bar baz);
///
/// assert_eq!("foo bar baz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L> FormatInto<L> for &Cow<'_, str>
where
    L: Lang,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        match self {
            Cow::Borrowed(b) => tokens.item(Item::Literal(ItemStr::from(b))),
            Cow::Owned(o) => o.format_into(tokens),
        }
    }
}

/// Cow slices are formatted by either borrowing or cloning the slice.
///
/// # Examples
///
/// ```
/// use std::borrow::Cow;
/// use genco::prelude::*;
///
/// let foo = Cow::<[&str]>::Borrowed(&["foo", "bar"]);
/// let bar = Cow::<[&str]>::Owned(vec!["baz"]);
///
/// let result: Tokens = quote!($foo $bar biz);
///
/// assert_eq!("foobar baz biz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L, T> FormatInto<L> for Cow<'_, [T]>
where
    L: Lang,
    T: FormatInto<L> + Clone,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        match self {
            Cow::Borrowed(b) => {
                for t in b.iter() {
                    tokens.append(t.clone());
                }
            }
            Cow::Owned(o) => o.format_into(tokens),
        }
    }
}

/// Cow slices are formatted by either borrowing or cloning the slice.
///
/// # Examples
///
/// ```
/// use std::borrow::Cow;
/// use genco::prelude::*;
///
/// let foo = &Cow::<[&str]>::Borrowed(&["foo", "bar"]);
/// let bar = &Cow::<[&str]>::Owned(vec!["baz"]);
///
/// let result: Tokens = quote!($foo $bar biz);
///
/// assert_eq!("foobar baz biz", result.to_string()?);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
impl<L, T> FormatInto<L> for &Cow<'_, [T]>
where
    L: Lang,
    T: FormatInto<L> + Clone,
{
    fn format_into(self, tokens: &mut Tokens<L>) {
        match self {
            Cow::Borrowed(b) => {
                for t in b.iter() {
                    tokens.append(t.clone());
                }
            }
            Cow::Owned(o) => o.format_into(tokens),
        }
    }
}

macro_rules! impl_display {
    ($($ty:ty),*) => {
        $(
            /// Implementation for primitive type. Uses the corresponding
            /// [Display][std::fmt::Display] implementation for the
            /// primitive type.
            impl<L> FormatInto<L> for $ty
            where
                L: Lang,
            {
                fn format_into(self, tokens: &mut Tokens<L>) {
                    tokens.append(self.to_string());
                }
            }
        )*
    };
}

impl_display!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize, usize);
