use std::io;

use crate::fmt;

/// Helper struct to format a token stream to an underlying writer implementing
/// [io::Write][std::io::Write].
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
/// use genco::fmt;
///
/// let map = rust::import("std::collections", "HashMap");
///
/// let tokens: rust::Tokens = quote! {
///     let mut m = $map::new();
///     m.insert(1u32, 2u32);
/// };
///
/// // Vec<u8> implements std::io::Write
/// let mut w = fmt::IoWriter::new(Vec::<u8>::new());
///
/// let fmt = fmt::Config::from_lang::<Rust>();
/// let config = rust::Config::default();
/// // Default format state for Rust.
/// let format = rust::Format::default();
///
/// tokens.format(&mut w.as_formatter(&fmt), &config, &format)?;
///
/// let vector = w.into_inner();
/// let string = std::str::from_utf8(&vector)?;
///
/// assert_eq!("let mut m = HashMap::new();\nm.insert(1u32, 2u32);", string);
/// # Ok::<_, anyhow::Error>(())
/// ```
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

    /// Convert into a formatter.
    pub fn as_formatter<'a>(&'a mut self, config: &'a fmt::Config) -> fmt::Formatter<'a> {
        fmt::Formatter::new(self, config)
    }

    /// Convert into the inner writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W> core::fmt::Write for IoWriter<W>
where
    W: io::Write,
{
    #[inline(always)]
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.writer
            .write_all(c.encode_utf8(&mut [0; 4]).as_bytes())
            .map_err(|_| core::fmt::Error)
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.writer
            .write_all(s.as_bytes())
            .map_err(|_| core::fmt::Error)
    }
}

impl<W> fmt::Write for IoWriter<W>
where
    W: io::Write,
{
    #[inline(always)]
    fn write_line(&mut self, config: &fmt::Config) -> fmt::Result {
        self.writer
            .write_all(config.newline.as_bytes())
            .map_err(|_| core::fmt::Error)
    }
}
