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

/// Trait to implement for language specialization.
pub trait Lang<'el>
where
    Self: Sized,
{
    /// Configuration associated with building a formatting element.
    type Config: Config;

    /// Format the language element.
    fn format(
        &self,
        _out: &mut Formatter,
        _config: &mut Self::Config,
        _level: usize,
    ) -> fmt::Result {
        Ok(())
    }

    /// Performing quoting according to convention set by custom element.
    fn quote_string(out: &mut Formatter, input: &str) -> fmt::Result {
        out.write_str(input)
    }

    /// Write a file according to convention by custom element.
    fn write_file(
        tokens: Tokens<'el, Self>,
        out: &mut Formatter,
        config: &mut Self::Config,
        level: usize,
    ) -> fmt::Result {
        tokens.format(out, config, level)
    }
}

/// Dummy implementation for unit.
impl<'el> Lang<'el> for () {
    type Config = ();
}
