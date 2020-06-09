use crate::formatter::Config;
use std::fmt;

/// Helper struct to write to an underlying writer.
#[derive(Default)]
pub(crate) struct VecWriter {
    line_buffer: String,
    target: Vec<String>,
}

impl VecWriter {
    /// Construct a new writer to a vector.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Convert into a vector.
    pub(crate) fn into_vec(mut self) -> Vec<String> {
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

impl crate::formatter::Write for VecWriter {
    #[inline(always)]
    fn write_line(&mut self, _: &Config) -> fmt::Result {
        self.target.push(self.line_buffer.clone());
        self.line_buffer.clear();
        Ok(())
    }
}
