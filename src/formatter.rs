use crate::Lang;

use std::fmt;
use std::io;
use std::iter;

/// Configuration to use for formatting output.
#[derive(Clone)]
pub struct FormatterConfig {
    /// Indentation level to use.
    indentation: usize,
    /// What to use as a newline.
    newline: &'static str,
}

impl FormatterConfig {
    /// Construct a new default formatter configuration for the specified
    /// language.
    pub fn from_lang<L>() -> FormatterConfig
    where
        L: Lang,
    {
        Self {
            indentation: L::default_indentation(),
            newline: "\n",
        }
    }

    /// Modify indentation to use.
    pub fn with_indentation(self, indentation: usize) -> Self {
        Self {
            indentation,
            ..self
        }
    }

    /// Set what to use as newline.
    pub fn with_newline(self, newline: &'static str) -> Self {
        Self { newline, ..self }
    }

    /// Current newline in use.
    #[inline(always)]
    pub fn newline(&self) -> &str {
        self.newline
    }
}

/// Trait that defines a line writer.
pub trait Write: fmt::Write {
    fn write_line(&mut self, config: &FormatterConfig) -> fmt::Result;
}

/// Helper struct to write to an underlying writer.
pub struct FmtWriter<W>
where
    W: fmt::Write,
{
    writer: W,
}

impl<W> FmtWriter<W>
where
    W: fmt::Write,
{
    /// Construct a new line writer from the underlying writer.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Convert into underlying writer.
    pub fn into_writer(self) -> W {
        self.writer
    }
}

impl<W> fmt::Write for FmtWriter<W>
where
    W: fmt::Write,
{
    #[inline(always)]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.writer.write_char(c)
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writer.write_str(s)
    }
}

impl<W> Write for FmtWriter<W>
where
    W: fmt::Write,
{
    #[inline(always)]
    fn write_line(&mut self, config: &FormatterConfig) -> fmt::Result {
        self.writer.write_str(config.newline())
    }
}

/// Helper struct to write to an underlying writer.
pub struct IoWriter<W>
where
    W: io::Write,
{
    writer: W,
}

impl<W> IoWriter<W>
where
    W: io::Write,
{
    /// Construct a new line writer from the underlying writer.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W> fmt::Write for IoWriter<W>
where
    W: io::Write,
{
    #[inline(always)]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.writer
            .write_all(c.encode_utf8(&mut [0; 4]).as_bytes())
            .map_err(|_| fmt::Error)
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writer.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

impl<W> Write for IoWriter<W>
where
    W: io::Write,
{
    #[inline(always)]
    fn write_line(&mut self, config: &FormatterConfig) -> fmt::Result {
        self.writer
            .write_all(config.newline().as_bytes())
            .map_err(|_| fmt::Error)
    }
}

/// Helper struct to write to an underlying writer.
#[derive(Default)]
pub struct VecWriter {
    line_buffer: String,
    target: Vec<String>,
}

impl VecWriter {
    /// Construct a new writer to a vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert into a vector.
    pub fn into_vec(mut self) -> Vec<String> {
        self.target.push(self.line_buffer);
        self.target
    }
}

impl fmt::Write for VecWriter {
    #[inline(always)]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.line_buffer.write_char(c)
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.line_buffer.write_str(s)
    }
}

impl Write for VecWriter {
    #[inline(always)]
    fn write_line(&mut self, _: &FormatterConfig) -> fmt::Result {
        self.target.push(self.line_buffer.clone());
        self.line_buffer.clear();
        Ok(())
    }
}

/// Formatter implementation for write types.
pub struct Formatter<'write> {
    write: &'write mut dyn Write,
    /// if last line was empty.
    current_line_empty: bool,
    /// Current indentation level.
    indent: usize,
    /// Number of indentations per level.
    pub(crate) config: FormatterConfig,
    /// Holds the current indentation level as a string.
    buffer: String,
}

impl<'write> Formatter<'write> {
    /// Create a new write formatter.
    pub fn new(write: &mut dyn Write, config: FormatterConfig) -> Formatter {
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
                .write_str(&self.buffer[0..(self.indent * self.config.indentation)])?;
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

        let extra = (self.indent * self.config.indentation).saturating_sub(self.buffer.len());

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
