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
    Element, FormatTokens, Formatter, FormatterConfig, Lang, LangBox, LangItem, VecWriter,
};
use std::collections::LinkedList;
use std::fmt;
use std::io;
use std::iter::FromIterator;
use std::result;
use std::vec;

/// A set of tokens.
#[derive(Debug, Default)]
pub struct Tokens<'el, L>
where
    L: Lang,
{
    pub(crate) elements: Vec<Element<'el, L>>,
}

/// Generic methods.
impl<'el, L> Tokens<'el, L>
where
    L: Lang,
{
    /// Create a new set of tokens.
    pub fn new() -> Tokens<'el, L> {
        Tokens {
            elements: Vec::new(),
        }
    }

    /// Push a nested definition.
    pub fn nested<T>(&mut self, tokens: T)
    where
        T: FormatTokens<'el, L>,
    {
        self.elements.push(Element::Indent);
        tokens.format_tokens(self);
        self.elements.push(Element::Unindent);
    }

    /// Push a nested definition.
    pub fn nested_into<B>(&mut self, builder: B) -> ()
    where
        B: FnOnce(&mut Tokens<'el, L>) -> (),
    {
        let mut t = Tokens::new();
        builder(&mut t);
        self.nested(t);
    }

    /// Push a nested definition.
    ///
    /// This is a fallible version that expected the builder to return a result.
    pub fn try_nested_into<E, B>(&mut self, builder: B) -> Result<(), E>
    where
        B: FnOnce(&mut Tokens<'el, L>) -> Result<(), E>,
    {
        let mut t = Tokens::new();
        builder(&mut t)?;
        self.nested(t);
        Ok(())
    }

    /// Push a nested reference to a definition.
    pub fn nested_ref(&mut self, tokens: &'el Tokens<'el, L>) {
        self.elements.push(Element::Indent);
        self.elements
            .extend(tokens.elements.iter().map(Element::Borrowed));
        self.elements.push(Element::Unindent);
    }

    /// Push a definition, guaranteed to be preceded with one newline.
    pub fn push<T>(&mut self, tokens: T)
    where
        T: FormatTokens<'el, L>,
    {
        self.elements.push(Element::PushSpacing);
        tokens.format_tokens(self);
    }

    /// Push a new created definition, guaranteed to be preceded with one newline.
    pub fn push_into<B>(&mut self, builder: B) -> ()
    where
        B: FnOnce(&mut Tokens<'el, L>) -> (),
    {
        let mut t = Tokens::new();
        builder(&mut t);
        self.push(t);
    }

    /// Push a new created definition, guaranteed to be preceded with one newline.
    ///
    /// This is a fallible version that expected the builder to return a result.
    pub fn try_push_into<E, B>(&mut self, builder: B) -> Result<(), E>
    where
        B: FnOnce(&mut Tokens<'el, L>) -> Result<(), E>,
    {
        let mut t = Tokens::new();
        builder(&mut t)?;
        self.push(t);
        Ok(())
    }

    /// Push the given set of tokens, unless it is empty.
    ///
    /// This is useful when you wish to preserve the structure of nested and joined tokens.
    pub fn push_unless_empty<T>(&mut self, tokens: T)
    where
        T: FormatTokens<'el, L>,
    {
        if !tokens.is_empty() {
            self.elements.push(Element::PushSpacing);
            tokens.format_tokens(self);
        }
    }

    /// Insert the given element.
    pub fn insert<E>(&mut self, pos: usize, element: E)
    where
        E: Into<Element<'el, L>>,
    {
        self.elements.insert(pos, element.into());
    }

    /// Append the given element.
    pub fn append<T>(&mut self, tokens: T)
    where
        T: FormatTokens<'el, L>,
    {
        tokens.format_tokens(self)
    }

    /// Append a reference to a definition.
    pub fn append_ref(&mut self, element: &'el Element<'el, L>) {
        self.elements.push(Element::Borrowed(element));
    }

    /// Append the given set of tokens, unless it is empty.
    ///
    /// This is useful when you wish to preserve the structure of nested and joined tokens.
    pub fn append_unless_empty<T>(&mut self, tokens: T)
    where
        T: FormatTokens<'el, L>,
    {
        if tokens.is_empty() {
            return;
        }

        tokens.format_tokens(self);
    }

    /// Extend with another set of tokens.
    pub fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Element<'el, L>>,
    {
        self.elements.extend(it.into_iter());
    }

    /// Walk over all elements.
    pub fn walk_custom(&self) -> WalkCustom<'_, 'el, L> {
        let mut queue = LinkedList::new();
        queue.extend(self.elements.iter());
        WalkCustom { queue: queue }
    }

    /// Add an registered custom element that is _not_ rendered.
    pub fn register(&mut self, custom: impl Into<LangBox<'el, L>>) {
        self.elements.push(Element::Registered(custom.into()));
    }

    /// Check if tokens contain no elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Add a single spacing to the token stream.
    pub fn spacing(&mut self) {
        self.elements.push(Element::Spacing);
    }

    /// Add a single line spacing to the token stream.
    pub fn line_spacing(&mut self) {
        self.elements.push(Element::LineSpacing);
    }

    /// Add a single push spacing operation.
    pub fn push_spacing(&mut self) {
        self.elements.push(Element::PushSpacing);
    }

    /// Add a single indentation to the token stream.
    pub fn indent(&mut self) {
        self.elements.push(Element::Indent);
    }

    /// Add a single unindentation to the token stream.
    pub fn unindent(&mut self) {
        self.elements.push(Element::Unindent);
    }
}

impl<'el, L> Clone for Tokens<'el, L>
where
    L: Lang,
{
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
        }
    }
}

impl<'el, L> IntoIterator for Tokens<'el, L>
where
    L: Lang,
{
    type Item = Element<'el, L>;
    type IntoIter = vec::IntoIter<Element<'el, L>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'el, L: Lang> Tokens<'el, L> {
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
            formatter.new_line_unless_empty()?;
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
            formatter.new_line_unless_empty()?;
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

impl<'el, C: Default, L: Lang<Config = C>> Tokens<'el, L> {
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

impl<'el, L> FromIterator<&'el Element<'el, L>> for Tokens<'el, L>
where
    L: Lang,
{
    fn from_iter<I: IntoIterator<Item = &'el Element<'el, L>>>(iter: I) -> Tokens<'el, L> {
        Tokens {
            elements: iter.into_iter().map(|e| Element::Borrowed(e)).collect(),
        }
    }
}

impl<'el, L> FromIterator<Element<'el, L>> for Tokens<'el, L>
where
    L: Lang,
{
    fn from_iter<I: IntoIterator<Item = Element<'el, L>>>(iter: I) -> Tokens<'el, L> {
        Tokens {
            elements: iter.into_iter().collect(),
        }
    }
}

pub struct WalkCustom<'a, 'el, L>
where
    L: Lang,
{
    queue: LinkedList<&'a Element<'el, L>>,
}

impl<'a, 'el, L> Iterator for WalkCustom<'a, 'el, L>
where
    L: Lang,
{
    type Item = &'a dyn LangItem<L>;

    fn next(&mut self) -> Option<Self::Item> {
        use self::Element::*;

        // read until custom element is encountered.
        while let Some(next) = self.queue.pop_front() {
            match *next {
                Rc(ref element) => {
                    self.queue.push_back(element.as_ref());
                }
                Borrowed(ref element) => {
                    self.queue.push_back(element);
                }
                LangBox(ref item) => return Some(&*item),
                Registered(ref item) => return Some(&*item),
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

    impl<'el> crate::Lang for Lang {
        type Config = ();
        type Import = Import;
    }

    #[test]
    fn test_walk_custom() {
        let mut toks: Tokens<Lang> = Tokens::new();

        toks.push(quote!(1:1 #(Import(1)) 1:2));

        // static string
        toks.append("bar");

        toks.nested(quote!(2:1 2:2 #(quote!(3:1 3:2)) #(Import(2))));

        // owned literal
        toks.append(String::from("nope"));

        let output: Vec<_> = toks
            .walk_custom()
            .flat_map(|import| import.as_import())
            .cloned()
            .collect();

        let expected = vec![Import(1), Import(2)];

        assert_eq!(expected, output);
    }
}
