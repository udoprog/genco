use crate::Lang;

/// Configuration to use for formatting output.
#[derive(Clone)]
pub struct Config {
    /// Indentation level to use.
    pub(super) indentation: usize,
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
}
