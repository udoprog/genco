//! A set of tokens that make up a single source-file.
//!
//! ## Example
//!
//! ```rust
//! use genco::{Tokens, Java};
//! let mut toks: Tokens<Java> = Tokens::new();
//! toks.append("foo");
//! ```

use crate::formatter::{FmtWriter, IoWriter};
use crate::{
    FormatTokens, Formatter, FormatterConfig, Item, Lang, LangItem, RegisterTokens, VecWriter,
};
use std::collections::LinkedList;
use std::fmt;
use std::io;
use std::iter::FromIterator;
use std::result;
use std::vec;

/// A set of tokens.
#[derive(Default)]
pub struct Tokens<L = ()>
where
    L: Lang,
{
    pub(crate) elements: Vec<Item<L>>,
}

impl<L> fmt::Debug for Tokens<L>
where
    L: Lang,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_list().entries(self.elements.iter()).finish()
    }
}

/// Generic methods.
impl<L> Tokens<L>
where
    L: Lang,
{
    /// Create a new set of tokens.
    pub fn new() -> Tokens<L> {
        Tokens {
            elements: Vec::new(),
        }
    }

    /// Append the given element.
    pub fn append<T>(&mut self, tokens: T)
    where
        T: FormatTokens<L>,
    {
        tokens.format_tokens(self)
    }

    /// Extend with another set of tokens.
    pub fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Item<L>>,
    {
        self.elements.extend(it.into_iter());
    }

    /// Walk over all elements.
    pub fn walk_custom(&self) -> WalkCustom<'_, L> {
        let mut queue = LinkedList::new();
        queue.extend(self.elements.iter());
        WalkCustom { queue: queue }
    }

    /// Add an registered custom element that is _not_ rendered.
    ///
    /// Registration can be used to generate imports that do not render a
    /// visible result.
    ///
    /// # Examples
    ///
    /// The `register` functionality is available through the [quote!] macro
    /// by using the [register] function.
    ///
    /// [register]: Tokens::register
    ///
    /// ```rust
    ///
    /// use genco::rust::{imported, Config};
    /// use genco::quote;
    ///
    /// let write_bytes_ext = imported("byteorder", "WriteBytesExt").alias("_");
    ///
    /// let tokens = quote!(#@(write_bytes_ext));
    ///
    /// assert_eq!("use byteorder::WriteBytesExt as _;\n\n", tokens.to_file_string().unwrap());
    /// ```
    ///
    /// [quote!]: genco_derive@quote!
    pub fn register<T>(&mut self, tokens: T)
    where
        T: RegisterTokens<L>,
    {
        tokens.register_tokens(self);
    }

    /// Check if tokens contain no elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Add a single spacing to the token stream.
    pub fn spacing(&mut self) {
        self.elements.push(Item::Spacing);
    }

    /// Add a single push spacing operation.
    pub fn push(&mut self) {
        // Already a push in the stream. Another one will do nothing.
        if let Some(Item::Push) = self.elements.last() {
            return;
        }

        self.elements.push(Item::Push);
    }

    /// Assert that there's the necessary elements to create one empty line at
    /// the top of the queue.
    pub fn push_line(&mut self) {
        let mut it = self.elements.iter().rev();

        let last = it.next();
        let ntl = it.next();

        match (ntl, last) {
            // A push + line is already at the end of the stream.
            (Some(Item::Push), Some(Item::Line)) => (),
            (_, Some(Item::Push)) => {
                self.elements.push(Item::Line);
            }
            // Assert that there is something to push behind us.
            (_, Some(..)) => {
                self.elements.push(Item::Push);
                self.elements.push(Item::Line);
            }
            // do nothing.
            _ => (),
        }
    }

    /// Add a single indentation to the token stream.
    pub fn indent(&mut self) {
        self.elements.push(Item::Indent);
    }

    /// Add a single unindentation to the token stream.
    pub fn unindent(&mut self) {
        self.elements.push(Item::Unindent);
    }
}

impl<L> Clone for Tokens<L>
where
    L: Lang,
{
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
        }
    }
}

impl<L> IntoIterator for Tokens<L>
where
    L: Lang,
{
    type Item = Item<L>;
    type IntoIter = vec::IntoIter<Item<L>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<L: Lang> Tokens<L> {
    /// Format the tokens.
    pub fn format(&self, out: &mut Formatter, config: &mut L::Config, level: usize) -> fmt::Result {
        for element in &self.elements {
            element.format(out, config, level)?;
        }

        Ok(())
    }

    /// Format the token stream as a file for the given target language to a
    /// string. Using the specified `config`.
    pub fn to_file_string_with(
        self,
        mut config: L::Config,
        format_config: FormatterConfig,
    ) -> result::Result<String, fmt::Error> {
        let mut w = FmtWriter::new(String::new());

        {
            let mut formatter = Formatter::new(&mut w, format_config);
            L::write_file(self, &mut formatter, &mut config, 0usize)?;
        }

        Ok(w.into_writer())
    }

    /// Format only the current token stream as a string. Using the specified
    /// `config`.
    pub fn to_string_with(
        self,
        mut config: L::Config,
        format_config: FormatterConfig,
    ) -> result::Result<String, fmt::Error> {
        let mut w = FmtWriter::new(String::new());

        {
            let mut formatter = Formatter::new(&mut w, format_config);
            self.format(&mut formatter, &mut config, 0usize)?;
        }

        Ok(w.into_writer())
    }

    /// Format tokens into a vector, where each entry equals a line in the
    /// resulting file. Using the specified `config`.
    pub fn to_file_vec_with(
        self,
        mut config: L::Config,
        format_config: FormatterConfig,
    ) -> result::Result<Vec<String>, fmt::Error> {
        let mut w = VecWriter::new();

        {
            let mut formatter = Formatter::new(&mut w, format_config);
            L::write_file(self, &mut formatter, &mut config, 0usize)?;
        }

        Ok(w.into_vec())
    }

    /// Format the token stream as a file for the given target language to the
    /// given `writer`. Using the specified `config`.
    pub fn to_fmt_writer_with<W>(
        self,
        writer: W,
        mut config: L::Config,
        format_config: FormatterConfig,
    ) -> result::Result<(), fmt::Error>
    where
        W: fmt::Write,
    {
        let mut w = FmtWriter::new(writer);

        {
            let mut formatter = Formatter::new(&mut w, format_config);
            L::write_file(self, &mut formatter, &mut config, 0usize)?;
            formatter.new_line_unless_empty()?;
        }

        Ok(())
    }

    /// Format the token stream as a file for the given target language to the
    /// given `writer`. Using the specified `config`.
    pub fn to_io_writer_with<W>(
        self,
        writer: W,
        mut config: L::Config,
        format_config: FormatterConfig,
    ) -> result::Result<(), fmt::Error>
    where
        W: io::Write,
    {
        let mut w = IoWriter::new(writer);

        {
            let mut formatter = Formatter::new(&mut w, format_config);
            L::write_file(self, &mut formatter, &mut config, 0usize)?;
            formatter.new_line_unless_empty()?;
        }

        Ok(())
    }
}

impl<C: Default, L: Lang<Config = C>> Tokens<L> {
    /// Format the token stream as a file for the given target language to a
    /// string. Using the default configuration.
    pub fn to_file_string(self) -> result::Result<String, fmt::Error> {
        self.to_file_string_with(L::Config::default(), FormatterConfig::from_lang::<L>())
    }

    /// Format only the current token stream as a string. Using the default
    /// configuration.
    pub fn to_string(self) -> result::Result<String, fmt::Error> {
        self.to_string_with(L::Config::default(), FormatterConfig::from_lang::<L>())
    }

    /// Format tokens into a vector, where each entry equals a line in the
    /// resulting file. Using the default configuration.
    pub fn to_file_vec(self) -> result::Result<Vec<String>, fmt::Error> {
        self.to_file_vec_with(L::Config::default(), FormatterConfig::from_lang::<L>())
    }

    /// Format the token stream as a file for the given target language to the
    /// given writer. Using the default configuration.
    pub fn to_fmt_writer<W>(self, writer: W) -> result::Result<(), fmt::Error>
    where
        W: fmt::Write,
    {
        self.to_fmt_writer_with(
            writer,
            L::Config::default(),
            FormatterConfig::from_lang::<L>(),
        )
    }

    /// Format the token stream as a file for the given target language to the
    /// given writer. Using the default configuration.
    pub fn to_io_writer<W>(self, writer: W) -> result::Result<(), fmt::Error>
    where
        W: io::Write,
    {
        self.to_io_writer_with(
            writer,
            L::Config::default(),
            FormatterConfig::from_lang::<L>(),
        )
    }
}

impl<'a, L> FromIterator<&'a Item<L>> for Tokens<L>
where
    L: Lang,
{
    fn from_iter<I: IntoIterator<Item = &'a Item<L>>>(iter: I) -> Tokens<L> {
        Tokens {
            elements: iter.into_iter().map(Clone::clone).collect(),
        }
    }
}

impl<L> FromIterator<Item<L>> for Tokens<L>
where
    L: Lang,
{
    fn from_iter<I: IntoIterator<Item = Item<L>>>(iter: I) -> Tokens<L> {
        Tokens {
            elements: iter.into_iter().collect(),
        }
    }
}

pub struct WalkCustom<'a, L>
where
    L: Lang,
{
    queue: LinkedList<&'a Item<L>>,
}

impl<'a, L> Iterator for WalkCustom<'a, L>
where
    L: Lang,
{
    type Item = &'a dyn LangItem<L>;

    fn next(&mut self) -> Option<Self::Item> {
        // read until custom element is encountered.
        while let Some(next) = self.queue.pop_front() {
            match next {
                Item::Rc(element) => {
                    self.queue.push_back(element.as_ref());
                }
                Item::LangBox(item) => return Some(&*item),
                Item::Registered(item) => return Some(&*item),
                _ => {}
            }
        }

        Option::None
    }
}

#[cfg(test)]
mod tests {
    use crate as genco;
    use crate::{quote, Formatter, LangItem, Tokens};
    use std::fmt;

    /// Own little custom language for this test.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Import(u32);

    impl_lang_item!(Import, Lang);

    impl LangItem<Lang> for Import {
        fn format(&self, out: &mut Formatter, _: &mut (), _: usize) -> fmt::Result {
            use std::fmt::Write as _;
            write!(out, "{}", self.0)
        }

        fn as_import(&self) -> Option<&Self> {
            Some(self)
        }
    }

    #[derive(Clone, Copy)]
    struct Lang(());

    impl crate::Lang for Lang {
        type Config = ();
        type Import = Import;
    }

    #[test]
    fn test_walk_custom() {
        let toks: Tokens<Lang> = quote! {
            1:1 #(Import(1)) 1:2
            bar
            2:1 2:2 #(quote!(3:1 3:2)) #(Import(2))
            #(String::from("nope"))
        };

        let output: Vec<_> = toks
            .walk_custom()
            .flat_map(|import| import.as_import())
            .cloned()
            .collect();

        let expected = vec![Import(1), Import(2)];

        assert_eq!(expected, output);
    }
}
