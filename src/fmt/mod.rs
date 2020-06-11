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
//! let map = rust::imported("std::collections", "HashMap");
//!
//! let tokens: rust::Tokens = quote! {
//!     let mut m = #map::new();
//!     m.insert(1u32, 2u32);
//! };
//!
//! let stdout = std::io::stdout();
//! let mut w = fmt::IoWriter::new(stdout.lock());
//!
//! let fmt_config = fmt::Config::from_lang::<Rust>().with_indentation(2);
//! let mut formatter = w.as_formatter(fmt_config);
//! let config = rust::Config::default();
//!
//! // Default format state for Rust.
//! let format = rust::Format::default();
//!
//! tokens.format(&mut formatter, &config, &format)?;
//! # Ok(())
//! # }
//! ```

use std::fmt;
use std::num::NonZeroI16;

mod config;
mod fmt_writer;
mod io_writer;
mod vec_writer;

pub use self::config::Config;
pub use self::fmt_writer::FmtWriter;
pub use self::io_writer::IoWriter;
pub use self::vec_writer::VecWriter;

/// Result type for the `fmt` module.
pub type Result<T = ()> = std::result::Result<T, std::fmt::Error>;

/// Buffer used as indentation source.
static INDENTATION: &str = "                                                                                                    ";

/// Trait that defines a line writer.
pub(crate) trait Write: std::fmt::Write {
    fn write_line(&mut self, config: &Config) -> Result;
}

/// Token stream formatter. Keeps track of everything we need to know in order
/// to enforce genco's indentation and whitespace rules.
pub struct Formatter<'a> {
    write: &'a mut (dyn Write + 'a),
    /// How many lines we want to add to the output stream.
    ///
    /// This will only be realized if we push non-whitespace.
    lines: usize,
    /// How many spaces we want to add to the output stream.
    ///
    /// This will only be realized if we push non-whitespace, and will be reset
    /// if a new line is pushed or indentation changes.
    spaces: usize,
    /// Current indentation level.
    indent: i16,
    /// Indicates if the line we are currently working on is empty or not.
    /// An empty line is one which is only prepared to add whitespace.
    line_empty: bool,
    /// Number of indentations per level.
    config: Config,
}

impl<'a> Formatter<'a> {
    /// Create a new write formatter.
    pub(crate) fn new(write: &'a mut (dyn Write + 'a), config: Config) -> Formatter<'a> {
        Formatter {
            write,
            lines: 0usize,
            spaces: 0usize,
            indent: 0i16,
            line_empty: true,
            config,
        }
    }

    /// Write the given string.
    pub(crate) fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > 0 {
            self.flush_whitespace()?;
            self.write.write_str(s)?;
        }

        Ok(())
    }

    pub(crate) fn push(&mut self) {
        if self.line_empty {
            return;
        }

        if self.lines < 1 {
            self.lines += 1;
        }

        self.line_empty = true;
    }

    /// Push a new line.
    pub(crate) fn line(&mut self) {
        self.line_empty = true;
        self.spaces = 0;

        // Limit the maximum number of lines to two.
        if self.lines < 2 {
            self.lines += 1;
        }
    }

    /// Push a space.
    pub(crate) fn space(&mut self) {
        self.spaces += 1;
    }

    /// Increase indentation level.
    pub(crate) fn indentation(&mut self, n: NonZeroI16) {
        if !self.line_empty {
            self.lines += 1;
            self.spaces = 0;
            self.line_empty = true;
        }

        self.indent += n.get();
    }

    // Realize any pending whitespace just prior to writing a non-whitespace
    // item.
    fn flush_whitespace(&mut self) -> fmt::Result {
        if std::mem::take(&mut self.line_empty) {
            for _ in 0..std::mem::take(&mut self.lines) {
                self.write.write_line(&self.config)?;
            }

            if self.indent > 0 {
                let mut to_write = self.indent as usize * self.config.indentation();

                while to_write > 0 {
                    let len = usize::min(to_write, INDENTATION.len());
                    self.write.write_str(&INDENTATION[0..len])?;
                    to_write -= len;
                }
            }
        }

        for _ in 0..std::mem::take(&mut self.spaces) {
            self.write.write_str(" ")?;
        }

        Ok(())
    }
}

impl<'a> fmt::Write for Formatter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > 0 {
            Formatter::write_str(self, s)?;
        }

        Ok(())
    }
}
