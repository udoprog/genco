use crate::formatter::Config;
use std::fmt;

/// Helper struct to write to an underlying writer.
pub(crate) struct FmtWriter<W>
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
    pub(crate) fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Convert into underlying writer.
    pub(crate) fn into_writer(self) -> W {
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

impl<W> crate::formatter::Write for FmtWriter<W>
where
    W: fmt::Write,
{
    #[inline(always)]
    fn write_line(&mut self, config: &Config) -> fmt::Result {
        self.writer.write_str(config.newline())
    }
}
