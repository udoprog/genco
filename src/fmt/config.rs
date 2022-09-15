use crate::lang::Lang;

/// Indentation configuration.
///
/// ```
/// use genco::prelude::*;
/// use genco::fmt;
///
/// let tokens: rust::Tokens = quote! {
///     fn foo() -> u32 {
///         42u32
///     }
/// };
///
/// let mut w = fmt::VecWriter::new();
///
/// let fmt = fmt::Config::from_lang::<Rust>()
///     .with_indentation(fmt::Indentation::Tab);
/// let config = rust::Config::default();
///
/// tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
///
/// assert_eq! {
///     vec![
///         "fn foo() -> u32 {",
///         "\t42u32",
///         "}",
///     ],
///     w.into_vec(),
/// };
/// # Ok::<_, genco::fmt::Error>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub enum Indentation {
    /// Each indentation is the given number of spaces.
    Space(usize),
    /// Each indentation is a tab.
    Tab,
}

/// Configuration to use for formatting output.
#[derive(Debug, Clone)]
pub struct Config {
    /// Indentation level to use.
    pub(super) indentation: Indentation,
    /// What to use as a newline.
    pub(super) newline: &'static str,
}

impl Config {
    /// Construct a new default formatter configuration for the specified
    /// language.
    pub fn from_lang<L>() -> Self
    where
        L: Lang,
    {
        Self {
            indentation: L::default_indentation(),
            newline: "\n",
        }
    }

    /// Modify indentation to use.
    pub fn with_indentation(self, indentation: Indentation) -> Self {
        Self {
            indentation,
            ..self
        }
    }

    /// Set what to use as newline.
    pub fn with_newline(self, newline: &'static str) -> Self {
        Self { newline, ..self }
    }
}
