//! Html

use core::fmt::Write as _;

use crate::fmt;

/// Tokens
pub type Tokens = crate::Tokens<Html>;

impl_lang! {
    /// The HTML language.
    pub Html {
        type Config = Config;
        type Format = Format;
        type Item = Item;

        #[inline]
        fn write_quoted(out: &mut fmt::Formatter<'_>, input: &str) -> fmt::Result {
            html_write_quoted(out, input)
        }

        #[inline]
        fn format_file(
            tokens: &Tokens,
            out: &mut fmt::Formatter<'_>,
            config: &Self::Config,
        ) -> fmt::Result {
            let format = Format::default();
            tokens.format(out, config, &format)
        }
    }

    Item(Item) {
        fn format(&self, _: &mut fmt::Formatter<'_>, _: &Config, _: &Format) -> fmt::Result {
            Ok(())
        }
    }
}

/// Formating configuration for HTML.
#[derive(Default)]
pub struct Format {}

/// An item in HTML.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {}

/// Html formatting configuration.
#[derive(Debug, Default)]
pub struct Config {}

/// Escape the given string according to HTML escape sequences.
pub fn html_write_quoted(out: &mut fmt::Formatter, input: &str) -> fmt::Result {
    for c in input.chars() {
        match c {
            '&' => out.write_str("&amp;")?,
            '<' => out.write_str("&lt;")?,
            '>' => out.write_str("&gt;")?,
            '"' => out.write_str("&quot;")?,
            '\'' => out.write_str("&apos;")?,
            c => out.write_char(c)?,
        };
    }

    Ok(())
}
