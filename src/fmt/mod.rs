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
//! let fmt_config = fmt::Config::from_lang::<Rust>()
//!     .with_indentation(fmt::Indentation::Space(2));
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
use std::mem;
use std::num::NonZeroI16;

mod config;
mod fmt_writer;
mod io_writer;
mod vec_writer;

pub use self::config::{Config, Indentation};
pub use self::fmt_writer::FmtWriter;
pub use self::io_writer::IoWriter;
pub use self::vec_writer::VecWriter;

/// Result type for the `fmt` module.
pub type Result<T = ()> = std::result::Result<T, std::fmt::Error>;
/// Error for the `fmt` module.
pub type Error = std::fmt::Error;

/// Buffer used as indentation source.
static SPACES: &str = "                                                                                                    ";

static TABS: &str =
    "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t";

/// Trait that defines a line writer.
pub(crate) trait Write: std::fmt::Write {
    fn write_line(&mut self, config: &Config) -> Result;
}

#[derive(Clone, Copy)]
enum Line {
    Initial,
    None,
    Push,
    Line,
}

impl Line {
    /// Convert into an indentation level.
    ///
    /// If we return `None`, no indentation nor lines should be written since we
    /// are at the initial stage of the file.
    fn into_indent(self) -> Option<usize> {
        match self {
            Self::Initial => Some(0),
            Self::Push => Some(1),
            Self::Line => Some(2),
            Self::None => return None,
        }
    }
}

impl Default for Line {
    fn default() -> Self {
        Self::None
    }
}

/// Token stream formatter. Keeps track of everything we need to know in order
/// to enforce genco's indentation and whitespace rules.
pub struct Formatter<'a> {
    write: &'a mut (dyn Write + 'a),
    /// How many lines we want to add to the output stream.
    ///
    /// This will only be realized if we push non-whitespace.
    line: Line,
    /// How many spaces we want to add to the output stream.
    ///
    /// This will only be realized if we push non-whitespace, and will be reset
    /// if a new line is pushed or indentation changes.
    spaces: usize,
    /// Current indentation level.
    indent: i16,
    /// Number of indentations per level.
    config: Config,
}

impl<'a> Formatter<'a> {
    /// Create a new write formatter.
    pub(crate) fn new(write: &'a mut (dyn Write + 'a), config: Config) -> Formatter<'a> {
        Formatter {
            write,
            line: Line::Initial,
            spaces: 0usize,
            indent: 0i16,
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
        self.line = match self.line {
            Line::Initial => return,
            Line::Line => return,
            _ => Line::Push,
        };

        self.spaces = 0;
    }

    /// Push a new line.
    pub(crate) fn line(&mut self) {
        self.line = match self.line {
            Line::Initial => return,
            _ => Line::Line,
        };

        self.spaces = 0;
    }

    /// Push a space.
    pub(crate) fn space(&mut self) {
        self.spaces += 1;
    }

    /// Increase indentation level.
    pub(crate) fn indentation(&mut self, n: NonZeroI16) {
        self.push();
        self.indent += n.get();
    }

    // Realize any pending whitespace just prior to writing a non-whitespace
    // item.
    fn flush_whitespace(&mut self) -> fmt::Result {
        let mut spaces = mem::take(&mut self.spaces);

        if let Some(lines) = mem::take(&mut self.line).into_indent() {
            for _ in 0..lines {
                self.write.write_line(&self.config)?;
            }

            let level = i16::max(self.indent, 0) as usize;

            match self.config.indentation {
                Indentation::Space(n) => {
                    spaces += level * n;
                }
                Indentation::Tab => {
                    let mut tabs = level;

                    while tabs > 0 {
                        let len = usize::min(tabs, TABS.len());
                        self.write.write_str(&TABS[0..len])?;
                        tabs -= len;
                    }
                }
            }
        }

        while spaces > 0 {
            let len = usize::min(spaces, SPACES.len());
            self.write.write_str(&SPACES[0..len])?;
            spaces -= len;
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
