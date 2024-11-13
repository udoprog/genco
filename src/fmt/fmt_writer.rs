use crate::fmt;

/// Helper struct to format a token stream to an underlying writer implementing
/// [fmt::Write][std::fmt::Write].
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
/// // Note: String implements std::fmt::Write
/// let mut w = fmt::FmtWriter::new(String::new());
///
/// let fmt = fmt::Config::from_lang::<Rust>();
///
/// let config = rust::Config::default();
/// // Default format state for Rust.
/// let format = rust::Format::default();
///
/// tokens.format(&mut w.as_formatter(&fmt), &config, &format)?;
///
/// let string = w.into_inner();
///
/// assert_eq!("let mut m = HashMap::new();\nm.insert(1u32, 2u32);", string);
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub struct FmtWriter<W>
where
    W: core::fmt::Write,
{
    writer: W,
}

impl<W> FmtWriter<W>
where
    W: core::fmt::Write,
{
    /// Construct a new line writer from the underlying writer.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Convert into a formatter.
    pub fn as_formatter<'a>(&'a mut self, config: &'a fmt::Config) -> fmt::Formatter<'a> {
        fmt::Formatter::new(self, config)
    }

    /// Convert into underlying writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W> core::fmt::Write for FmtWriter<W>
where
    W: core::fmt::Write,
{
    #[inline(always)]
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.writer.write_char(c)
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.writer.write_str(s)
    }
}

impl<W> fmt::Write for FmtWriter<W>
where
    W: core::fmt::Write,
{
    #[inline(always)]
    fn write_line(&mut self, config: &fmt::Config) -> fmt::Result {
        self.writer.write_str(config.newline)
    }
}
