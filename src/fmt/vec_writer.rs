use crate::fmt;

/// Helper struct to format a token stream as a vector of strings.
///
/// # Examples
///
/// ```rust
/// use genco::prelude::*;
/// use genco::fmt;
///
/// # fn main() -> fmt::Result {
/// let map = rust::import("std::collections", "HashMap");
///
/// let tokens: rust::Tokens = quote! {
///     let mut m = #map::new();
///     m.insert(1u32, 2u32);
/// };
///
/// // Note: String implements std::fmt::Write
/// let mut w = fmt::VecWriter::new();
///
/// let fmt_config = fmt::Config::from_lang::<Rust>();
/// let mut formatter = w.as_formatter(fmt_config);
///
/// let config = rust::Config::default();
/// // Default format state for Rust.
/// let format = rust::Format::default();
///
/// tokens.format(&mut formatter, &config, &format)?;
///
/// let vec = w.into_vec();
///
/// assert_eq!(
///     vec![
///         "let mut m = HashMap::new();",
///         "m.insert(1u32, 2u32);",
///     ],
///     vec
/// );
/// # Ok(())
/// # }
/// ```
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

    /// Convert into a formatter.
    pub fn as_formatter(&mut self, config: fmt::Config) -> fmt::Formatter<'_> {
        fmt::Formatter::new(self, config)
    }

    /// Convert into a vector.
    pub fn into_vec(mut self) -> Vec<String> {
        self.target.push(self.line_buffer);
        self.target
    }
}

impl std::fmt::Write for VecWriter {
    #[inline(always)]
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.line_buffer.write_char(c)
    }

    #[inline(always)]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.line_buffer.write_str(s)
    }
}

impl fmt::Write for VecWriter {
    #[inline(always)]
    fn write_line(&mut self, _: &fmt::Config) -> fmt::Result {
        self.target.push(self.line_buffer.clone());
        self.line_buffer.clear();
        Ok(())
    }

    // NB: trailing line is ignored for vector writer.
    fn write_trailing_line(&mut self, _: &fmt::Config) -> fmt::Result {
        Ok(())
    }
}
