use crate::formatter::Config;

use std::fmt;
use std::io;

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

impl<W> crate::formatter::Write for IoWriter<W>
where
    W: io::Write,
{
    #[inline(always)]
    fn write_line(&mut self, config: &Config) -> fmt::Result {
        self.writer
            .write_all(config.newline().as_bytes())
            .map_err(|_| fmt::Error)
    }
}
