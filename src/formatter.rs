use std::fmt;
use std::iter;

/// Helper trait to format tokens.
pub trait Formatter: fmt::Write {
    /// Forcibly create a new line.
    fn new_line(&mut self) -> fmt::Result;

    /// Create a new line unless the current line is empty.
    fn new_line_unless_empty(&mut self) -> fmt::Result;

    /// Indent the formatter.
    fn indent(&mut self);

    /// Unindent the formatter.
    fn unindent(&mut self);
}

/// Formatter implementation for write types.
pub struct WriteFormatter<'write, W>
where
    W: fmt::Write + 'write,
{
    write: &'write mut W,
    /// if last line was empty.
    last_line_empty: bool,
    /// Current indentation level.
    indent: usize,
    /// Holds the current indentation level as a string.
    indent_buffer: String,
}

impl<'write, W> WriteFormatter<'write, W>
where
    W: fmt::Write,
{
    /// Create a new write formatter.
    pub fn new(write: &mut W) -> WriteFormatter<W> {
        WriteFormatter {
            write: write,
            last_line_empty: true,
            indent: 0usize,
            indent_buffer: String::from("  "),
        }
    }

    fn check_indent(&mut self) -> fmt::Result {
        if self.last_line_empty {
            self.write.write_str(
                &self.indent_buffer[0..self.indent * 2],
            )?;
        }

        self.last_line_empty = false;
        Ok(())
    }
}

impl<'write, W> fmt::Write for WriteFormatter<'write, W>
where
    W: fmt::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.check_indent()?;
        self.write.write_str(s)
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.check_indent()?;
        self.write.write_char(c)
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        self.check_indent()?;
        self.write.write_fmt(args)
    }
}

impl<'write, W> Formatter for WriteFormatter<'write, W>
where
    W: fmt::Write,
{
    fn new_line(&mut self) -> fmt::Result {
        self.write.write_char('\n')?;
        self.last_line_empty = true;
        Ok(())
    }

    fn new_line_unless_empty(&mut self) -> fmt::Result {
        if !self.last_line_empty {
            self.write.write_char('\n')?;
            self.last_line_empty = true;
        }

        Ok(())
    }

    fn indent(&mut self) {
        self.indent += 1;

        // check that buffer contains the current indentation.
        if self.indent_buffer.len() < self.indent * 2 {
            // double the buffer
            for c in iter::repeat(' ').take(self.indent_buffer.len()) {
                self.indent_buffer.push(c);
            }
        }
    }

    fn unindent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }
}
