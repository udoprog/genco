//! Specialization for HTML code generation.
//!
//! Template text and interpolated values are emitted verbatim. Only values
//! passed through [`quoted()`] or [`text()`] are escaped:
//!
//! * [`quoted()`] produces a double-quoted attribute value, escaping `&`, `<`,
//!   `>`, `"` and `'`.
//! * [`text()`] produces element content, escaping `&`, `<` and `>`.
//!
//! [`quoted()`]: crate::tokens::quoted()
//!
//! # Examples
//!
//! ```rust
//! use genco::prelude::*;
//!
//! let url = "/search?q=a&b";
//! let title = "Tom & Jerry <3";
//!
//! let toks: html::Tokens = quote! {
//!     <a href=$(quoted(url))>$(html::text(title))</a>
//! };
//!
//! assert_eq!(
//!     vec![
//!         "<a href=\"/search?q=a&amp;b\">Tom &amp; Jerry &lt;3</a>",
//!     ],
//!     toks.to_file_vec()?
//! );
//! # Ok::<_, genco::fmt::Error>(())
//! ```

use core::fmt::Write as _;

use alloc::string::String;

use crate::fmt;
use crate::tokens::{FormatInto, ItemStr};

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
            match *self {}
        }
    }
}

/// Formating configuration for HTML.
#[derive(Default)]
pub struct Format {}

/// Language items for HTML.
///
/// HTML has no imports or other language items, so this type is uninhabited.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {}

/// Html formatting configuration.
#[derive(Debug, Default)]
pub struct Config {}

/// Escape the given string according to HTML escape sequences.
///
/// This escapes `&`, `<`, `>`, `"` and `'`, and is what [`quoted()`] uses for
/// HTML.
///
/// [`quoted()`]: crate::tokens::quoted()
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

/// Escape text for use as HTML element content.
///
/// This escapes `&`, `<` and `>`. Use [`quoted()`] for attribute values
/// instead, since it also escapes quotes.
///
/// [`quoted()`]: crate::tokens::quoted()
///
/// # Examples
///
/// ```
/// use genco::prelude::*;
///
/// let toks: html::Tokens = quote! {
///     <p>$(html::text("1 < 2 && \"3\" > 2"))</p>
/// };
///
/// assert_eq!(
///     vec!["<p>1 &lt; 2 &amp;&amp; \"3\" &gt; 2</p>"],
///     toks.to_file_vec()?
/// );
/// # Ok::<_, genco::fmt::Error>(())
/// ```
pub fn text<T>(text: T) -> Text<T>
where
    T: Into<ItemStr>,
{
    Text { text }
}

/// Text escaped as HTML element content.
///
/// This is constructed with the [text()] function.
#[derive(Debug, Clone, Copy)]
pub struct Text<T> {
    text: T,
}

impl<T> FormatInto<Html> for Text<T>
where
    T: Into<ItemStr>,
{
    fn format_into(self, tokens: &mut Tokens) {
        let text = self.text.into();

        if !text.contains(['&', '<', '>']) {
            tokens.literal(text);
            return;
        }

        let mut escaped = String::with_capacity(text.len());

        for c in text.chars() {
            match c {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                c => escaped.push(c),
            }
        }

        tokens.literal(escaped);
    }
}
