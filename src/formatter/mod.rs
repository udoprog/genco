use std::fmt;
use std::iter;

mod config;
mod fmt_writer;
mod io_writer;
mod vec_writer;

pub use self::config::Config;
pub(crate) use self::fmt_writer::FmtWriter;
pub(crate) use self::io_writer::IoWriter;
pub(crate) use self::vec_writer::VecWriter;

/// Trait that defines a line writer.
pub trait Write: fmt::Write {
    fn write_line(&mut self, config: &Config) -> fmt::Result;
}

/// Formatter implementation for write types.
pub struct Formatter<'write> {
    write: &'write mut dyn Write,
    /// if last line was empty.
    current_line_empty: bool,
    /// Current indentation level.
    indent: usize,
    /// Number of indentations per level.
    config: Config,
    /// Holds the current indentation level as a string.
    buffer: String,
}

impl<'write> Formatter<'write> {
    /// Create a new write formatter.
    pub fn new(write: &mut dyn Write, config: Config) -> Formatter {
        Formatter {
            write: write,
            current_line_empty: true,
            indent: 0usize,
            config,
            buffer: String::from("  "),
        }
    }

    fn check_indent(&mut self) -> fmt::Result {
        if self.current_line_empty && self.indent > 0 {
            self.write
                .write_str(&self.buffer[0..(self.indent * self.config.indentation())])?;
            self.current_line_empty = false;
        }

        Ok(())
    }

    /// Write the given string.
    pub fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > 0 {
            self.check_indent()?;
            self.write.write_str(s)?;
            self.current_line_empty = false;
        }

        Ok(())
    }

    /// Push a new line.
    pub fn new_line(&mut self) -> fmt::Result {
        self.write.write_line(&self.config)?;
        self.current_line_empty = true;
        Ok(())
    }

    /// Push a new line, unless the current line is empty.
    pub fn new_line_unless_empty(&mut self) -> fmt::Result {
        if !self.current_line_empty {
            self.new_line()?;
        }

        Ok(())
    }

    /// Increase indentation level.
    pub fn indent(&mut self) {
        self.indent += 1;

        let extra = (self.indent * self.config.indentation()).saturating_sub(self.buffer.len());

        // check that buffer contains the current indentation.
        for c in iter::repeat(' ').take(extra) {
            self.buffer.push(c);
        }
    }

    /// Decrease indentation level.
    pub fn unindent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }
}

impl<'write> fmt::Write for Formatter<'write> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > 0 {
            self.check_indent()?;
            self.write.write_str(s)?;
            self.current_line_empty = false;
        }

        Ok(())
    }
}
