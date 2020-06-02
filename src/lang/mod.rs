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

use crate::{Config, Formatter, Tokens};
use std::fmt;
use std::rc::Rc;

/// Trait to implement for language specialization.
pub trait Lang
where
    Self: Sized,
{
    /// Configuration associated with building a formatting element.
    type Config: Config;
    /// The type used when resolving imports.
    type Import;

    /// Performing quoting according to convention set by custom element.
    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_str(input)
    }

    /// Write a file according to convention by custom element.
    fn write_file(
        tokens: Tokens<'_, Self>,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        tokens.format(out, config, level)
    }
}

/// Dummy implementation for unit.
impl<'el> Lang for () {
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
    L: Lang,
{
    /// Format the language item appropriately.
    fn format(&self, out: &mut Formatter, config: &mut L::Config, level: usize) -> fmt::Result;

    /// Coerce into an imported type.
    ///
    /// This is used for import resolution for custom language items.
    fn as_import(&self) -> Option<&L::Import>;
}

/// A box containing a lang item.
pub enum LangBox<'el, L>
where
    L: Lang,
{
    /// A reference-counted dynamic language item.
    Rc(Rc<dyn LangItem<L>>),
    /// A reference to a dynamic language item.
    Ref(&'el dyn LangItem<L>),
}

impl<'el, L> Clone for LangBox<'el, L>
where
    L: Lang,
{
    fn clone(&self) -> Self {
        match self {
            Self::Rc(lang) => Self::Rc(lang.clone()),
            Self::Ref(lang) => Self::Ref(*lang),
        }
    }
}

impl<'el, L> fmt::Debug for LangBox<'el, L>
where
    L: Lang,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "LangBox")
    }
}

impl<'el, L> LangItem<L> for LangBox<'el, L>
where
    L: Lang,
{
    fn format(&self, out: &mut Formatter, config: &mut L::Config, level: usize) -> fmt::Result {
        match self {
            Self::Rc(this) => this.format(out, config, level),
            Self::Ref(this) => this.format(out, config, level),
        }
    }

    fn as_import(&self) -> Option<&L::Import> {
        match self {
            Self::Rc(this) => this.as_import(),
            Self::Ref(this) => this.as_import(),
        }
    }
}
