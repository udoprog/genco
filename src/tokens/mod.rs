//! Utilities for working with token streams.
//!
//! This is typically a module you will use if you intend to provide a manual
//! implementation of [FormatInto].
//!
//! # Examples
//!
//! ```rust
//! use genco::quote_in;
//! use genco::tokens::{from_fn, ItemStr, FormatInto, static_literal};
//! use genco::lang::Lang;
//!
//! /// Format a block comment, starting with `/**`, and ending in `*/`.
//! pub fn block_comment<I, L>(text: I) -> impl FormatInto<L>
//! where
//!     I: IntoIterator,
//!     I::Item: Into<ItemStr>,
//!     L: Lang,
//! {
//!     from_fn(move |t| {
//!         let mut it = text.into_iter().peekable();
//!
//!         if it.peek().is_some() {
//!             quote_in! { *t =>
//!                 $(static_literal("/**"))
//!                 $(for line in it join ($['\r']) {
//!                     $[' ']* $(line.into())
//!                 })
//!                 $[' ']$(static_literal("*/"))
//!             }
//!         }
//!     })
//! }
//!
//! # fn main() -> genco::fmt::Result {
//! use genco::prelude::*;
//!
//! let tokens: java::Tokens = quote! {
//!     $(block_comment(&["This class is used for awesome stuff", "ok?"]))
//!     public static class Foo {
//!     }
//! };
//!
//! assert_eq!(
//!     vec![
//!         "/**",
//!         " * This class is used for awesome stuff",
//!         " * ok?",
//!         " */",
//!         "public static class Foo {",
//!         "}"
//!     ],
//!     tokens.to_vec()?
//! );
//! # Ok(())
//! # }
//! ```

mod display;
mod format_into;
mod from_fn;
mod internal;
mod item;
mod item_str;
mod quoted;
mod register;
mod static_literal;
mod tokens;

pub use self::display::{display, Display};
pub use self::format_into::FormatInto;
pub use self::from_fn::{from_fn, FromFn};
pub use self::item::Item;
pub use self::item_str::ItemStr;
pub use self::quoted::{quoted, QuotedFn};
pub use self::register::{register, Register, RegisterFn};
pub use self::static_literal::static_literal;
pub use self::tokens::Tokens;

#[doc(hidden)]
pub use self::internal::__lang_item;
#[doc(hidden)]
pub use self::internal::__lang_item_register;
