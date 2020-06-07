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

use crate::{Formatter, Tokens};
use std::any::Any;
use std::fmt;
use std::rc::Rc;

/// Trait to implement for language specialization.
pub trait Lang
where
    Self: 'static + Sized,
{
    /// Configuration associated with building a formatting element.
    type Config;
    /// The type used when resolving imports.
    type Import: ?Sized;

    /// The default indentation for the current language.
    fn default_indentation() -> usize {
        4
    }

    /// Performing quoting according to convention set by custom element.
    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_str(input)
    }

    /// Write a file according to convention by custom element.
    fn write_file(
        tokens: Tokens<Self>,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        tokens.format(out, config, level)
    }
}

/// Dummy implementation for unit.
impl Lang for () {
    type Config = ();
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
    fn format(&self, out: &mut Formatter, config: &mut L::Config, level: usize) -> fmt::Result;

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
pub struct LangBox<L> {
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

impl<L> fmt::Debug for LangBox<L>
where
    L: Lang,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "LangBox")
    }
}

impl<L> LangItem<L> for LangBox<L>
where
    L: Lang,
{
    fn format(&self, out: &mut Formatter, config: &mut L::Config, level: usize) -> fmt::Result {
        self.inner.format(out, config, level)
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
