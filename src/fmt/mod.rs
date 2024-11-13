//! Code formatting utilities.
//!
//! So you have a token stream and it's time to format it into a
//! file/string/whatever? You've come to the right place!
//!
//! Formatting is done through the following utilities:
//!
//! * [fmt::VecWriter][VecWriter] - To write result into a vector.
//! * [fmt::FmtWriter][FmtWriter] - To write the result into something
//!   implementing [fmt::Write][std::fmt::Write].
//! * [fmt::IoWriter][IoWriter]- To write the result into something implementing
//!   [io::Write][std::io::Write].
//!
//! # Examples
//!
//! The following is an example, showcasing how you can format directly to
//! [stdout].
//!
//! [stdout]: std::io::stdout
//!
//! # Examples
//!
//! ```rust,no_run
//! use genco::prelude::*;
//! use genco::fmt;
//!
//! # fn main() -> fmt::Result {
//! let map = rust::import("std::collections", "HashMap");
//!
//! let tokens: rust::Tokens = quote! {
//!     let mut m = #map::new();
//!     m.insert(1u32, 2u32);
//! };
//!
//! let stdout = std::io::stdout();
//! let mut w = fmt::IoWriter::new(stdout.lock());
//!
//! let fmt = fmt::Config::from_lang::<Rust>()
//!     .with_indentation(fmt::Indentation::Space(2));
//! let config = rust::Config::default();
//!
//! // Default format state for Rust.
//! let format = rust::Format::default();
//!
//! tokens.format(&mut w.as_formatter(&fmt), &config, &format)?;
//! # Ok(())
//! # }
//! ```

mod config;
mod cursor;
mod fmt_writer;
mod formatter;
#[cfg(feature = "std")]
mod io_writer;
mod vec_writer;

pub use self::config::{Config, Indentation};
pub use self::fmt_writer::FmtWriter;
pub use self::formatter::Formatter;
#[cfg(feature = "std")]
pub use self::io_writer::IoWriter;
pub use self::vec_writer::VecWriter;

/// Result type for the `fmt` module.
pub type Result<T = ()> = core::result::Result<T, core::fmt::Error>;
/// Error for the `fmt` module.
pub type Error = core::fmt::Error;

/// Trait that defines a line writer.
pub(crate) trait Write: core::fmt::Write {
    /// Implement for writing a line.
    fn write_line(&mut self, config: &Config) -> Result;

    /// Implement for writing the trailing line ending of the file.
    #[inline]
    fn write_trailing_line(&mut self, config: &Config) -> Result {
        self.write_line(config)
    }
}
