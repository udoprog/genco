//! Language specialization for genco
//!
//! This module contains sub-modules which provide implementations of the [Lang]
//! trait to configure genco for various programming languages.
//!
//! This module also provides a dummy [Lang] implementation for `()`.
//!
//! This allows `()` to be used as a quick and dirty way to do formatting,
//! usually for examples.
//!
//! ```rust
//! use genco::prelude::*;
//!
//! # fn main() -> genco::fmt::Result {
//! let tokens: Tokens = quote!(hello world);
//! # Ok(())
//! # }
//! ```

pub mod csharp;
pub mod dart;
pub mod go;
pub mod java;
pub mod js;
pub mod python;
pub mod rust;
pub mod swift;

pub use self::csharp::Csharp;
pub use self::dart::Dart;
pub use self::go::Go;
pub use self::java::Java;
pub use self::js::JavaScript;
pub use self::python::Python;
pub use self::rust::Rust;
pub use self::swift::Swift;

use crate::fmt;
use crate::Tokens;
use std::any::Any;
use std::rc::Rc;

/// Trait to implement for language specialization.
///
/// The various language implementations can be found in the [lang][self]
/// module.
pub trait Lang
where
    Self: 'static + Sized,
{
    /// Configuration associated with building a formatting element.
    type Config;
    /// State being used during formatting.
    type Format: Default;
    /// The type used when resolving imports.
    type Import: ?Sized;

    /// Provide the default indentation.
    fn default_indentation() -> fmt::Indentation {
        fmt::Indentation::Space(4)
    }

    /// Performing string quoting according to language convention.
    fn quote_string(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
        out.write_str(input)
    }

    /// Write a file according to the specified language convention.
    fn format_file(
        tokens: &Tokens<Self>,
        out: &mut fmt::Formatter<'_>,
        config: &Self::Config,
    ) -> fmt::Result {
        let format = Self::Format::default();
        tokens.format(out, config, &format)
    }
}

impl Lang for () {
    type Config = ();
    type Format = ();
    type Import = ();
}

/// A type-erased holder for language-specific items.
///
/// Carries formatting and coercion functions like
/// [as_import][LangItem::as_import] to allow language specific processing to
/// work.
pub trait LangItem<L>
where
    Self: Any,
    L: Lang,
{
    /// Format the language item appropriately.
    fn format(
        &self,
        out: &mut fmt::Formatter<'_>,
        config: &L::Config,
        format: &L::Format,
    ) -> fmt::Result;

    /// Check equality.
    fn eq(&self, other: &dyn LangItem<L>) -> bool;

    /// Convert into any type.
    fn as_any(&self) -> &dyn Any;

    /// Coerce into an imported type.
    ///
    /// This is used for import resolution for custom language items.
    fn as_import(&self) -> Option<&L::Import> {
        None
    }
}

/// A box containing a lang item.
pub struct LangBox<L>
where
    L: Lang,
{
    inner: Rc<dyn LangItem<L>>,
}

impl<L> Clone for LangBox<L>
where
    L: Lang,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<L> std::fmt::Debug for LangBox<L>
where
    L: Lang,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "LangBox")
    }
}

impl<L> LangItem<L> for LangBox<L>
where
    L: Lang,
{
    fn format(
        &self,
        out: &mut fmt::Formatter<'_>,
        config: &L::Config,
        format: &L::Format,
    ) -> fmt::Result {
        self.inner.format(out, config, format)
    }

    fn eq(&self, other: &dyn LangItem<L>) -> bool {
        self.inner.eq(other)
    }

    fn as_any(&self) -> &dyn Any {
        self.inner.as_any()
    }

    fn as_import(&self) -> Option<&L::Import> {
        self.inner.as_import()
    }
}

impl<L> From<Rc<dyn LangItem<L>>> for LangBox<L>
where
    L: Lang,
{
    fn from(value: Rc<dyn LangItem<L>>) -> Self {
        Self { inner: value }
    }
}
