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

    /// Start a string quote.
    fn open_quote(
        out: &mut fmt::Formatter<'_>,
        _config: &Self::Config,
        _format: &Self::Format,
        _has_eval: bool,
    ) -> fmt::Result {
        use std::fmt::Write as _;
        out.write_char('"')?;
        Ok(())
    }

    /// End a string quote.
    fn close_quote(
        out: &mut fmt::Formatter<'_>,
        _config: &Self::Config,
        _format: &Self::Format,
        _has_eval: bool,
    ) -> fmt::Result {
        use std::fmt::Write as _;
        out.write_char('"')?;
        Ok(())
    }

    /// A simple, single-literal string evaluation.
    fn string_eval_literal(
        out: &mut fmt::Formatter<'_>,
        config: &Self::Config,
        format: &Self::Format,
        literal: &str,
    ) -> fmt::Result {
        Self::start_string_eval(out, config, format)?;
        out.write_str(literal)?;
        Self::end_string_eval(out, config, format)?;
        Ok(())
    }

    /// Start a string-interpolated eval.
    fn start_string_eval(
        _out: &mut fmt::Formatter<'_>,
        _config: &Self::Config,
        _format: &Self::Format,
    ) -> fmt::Result {
        Ok(())
    }

    /// End a string interpolated eval.
    fn end_string_eval(
        _out: &mut fmt::Formatter<'_>,
        _config: &Self::Config,
        _format: &Self::Format,
    ) -> fmt::Result {
        Ok(())
    }

    /// Performing string quoting according to language convention.
    fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
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

/// Marker trait to indicate that the given language supports string evaluations.
pub trait LangSupportsEval: Lang {}

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
    Self: Any + std::fmt::Debug,
    L: Lang,
{
    /// Format the language item appropriately.
    fn format(&self, _: &mut fmt::Formatter<'_>, _: &L::Config, _: &L::Format) -> fmt::Result {
        Ok(())
    }

    /// LangItem convert to Any. Automatically implemented by macro.
    fn __lang_item_as_any(&self) -> &dyn Any;

    /// LangItem clone. Automatically implemented by macro.
    fn __lang_item_clone(&self) -> Box<dyn LangItem<L>>;

    /// LangItem equality. Automatically implemented by macro.
    fn __lang_item_eq(&self, other: &dyn LangItem<L>) -> bool;

    /// Coerce into an imported type.
    ///
    /// This is used for import resolution for custom language items.
    fn as_import(&self) -> Option<&L::Import> {
        None
    }
}

/// Escape the given string according to a C-family escape sequence.
///
/// See https://en.wikipedia.org/wiki/Escape_sequences_in_C
///
/// This is one of the more common escape sequences and is provided here so you
/// can use it if a language you've implemented requires it.
pub fn c_family_write_quoted(out: &mut fmt::Formatter, input: &str) -> fmt::Result {
    use std::fmt::Write as _;

    for c in input.chars() {
        match c {
            // alert (bell)
            '\u{0007}' => out.write_str("\\a")?,
            // backspace
            '\u{0008}' => out.write_str("\\b")?,
            // form feed
            '\u{0012}' => out.write_str("\\f")?,
            // new line
            '\n' => out.write_str("\\n")?,
            // carriage return
            '\r' => out.write_str("\\r")?,
            // horizontal tab
            '\t' => out.write_str("\\t")?,
            // vertical tab
            '\u{0011}' => out.write_str("\\v")?,
            '\'' => out.write_str("\\'")?,
            '"' => out.write_str("\\\"")?,
            '\\' => out.write_str("\\\\")?,
            ' ' => out.write_char(' ')?,
            c if c.is_ascii() => {
                if !c.is_control() {
                    out.write_char(c)?
                } else {
                    write!(out, "\\x{:02x}", c as u32)?;
                }
            }
            c if (c as u32) < 0x10000 => {
                write!(out, "\\u{:04x}", c as u32)?;
            }
            c => {
                write!(out, "\\U{:08x}", c as u32)?;
            }
        };
    }

    Ok(())
}
