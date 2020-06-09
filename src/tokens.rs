//! A set of tokens that make up a single source-file.
//!
//! ## Example
//!
//! ```rust
//! use genco::{Tokens, Java};
//! let mut toks: Tokens<Java> = Tokens::new();
//! toks.append("foo");
//! ```

use crate::formatter::{FmtWriter, IoWriter, VecWriter};
use crate::{FormatTokens, Formatter, FormatterConfig, Item, Lang, LangItem, RegisterTokens};
use std::cmp;
use std::fmt;
use std::io;
use std::iter::FromIterator;
use std::num::NonZeroI16;
use std::result;
use std::slice;
use std::vec;

/// A stream of tokens.
///
/// # Structural Requirements
///
/// While not strictly necessary, this structure does its best to maintain
/// so-called structural requirements.
///
/// That means the following:
///
/// * Only one [space()] may occur in sequence.
/// * Only one [push()] may occur in sequence.
/// * A [push()] may never be preceeded by a [line()], since it would have no
///   effect.
/// * Every [line()] must be preceeded by a [push()].
///
/// ```rust
/// use genco::{Tokens, Item};
///
/// let mut tokens = Tokens::<()>::new();
///
/// tokens.push();
/// tokens.push();
///
/// assert_eq!(vec![Item::Push::<()>], tokens);
/// ```
#[derive(Default)]
pub struct Tokens<L = ()>
where
    L: Lang,
{
    items: Vec<Item<L>>,
}

impl<L> fmt::Debug for Tokens<L>
where
    L: Lang,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_list().entries(self.items.iter()).finish()
    }
}

/// Generic methods.
impl<L> Tokens<L>
where
    L: Lang,
{
    /// Create a new set of tokens.
    pub fn new() -> Tokens<L> {
        Tokens { items: Vec::new() }
    }

    /// Construct an iterator over this token stream.
    pub fn iter(&self) -> Iter<'_, L> {
        Iter {
            iter: self.items.iter(),
        }
    }

    /// Construct an iterator that owns all items in token stream.
    pub fn into_iter(self) -> IntoIter<L> {
        IntoIter {
            iter: self.items.into_iter(),
        }
    }

    /// Append the given element.
    pub fn append<T>(&mut self, tokens: T)
    where
        T: FormatTokens<L>,
    {
        tokens.format_tokens(self)
    }

    /// Append a single item to the stream, while checking for structural
    /// guarantees.
    pub fn push_item(&mut self, item: Item<L>) {
        match item {
            Item::Push => self.push(),
            Item::Line => self.line(),
            Item::Space => self.space(),
            other => self.items.push(other),
        }
    }

    /// Extend with another set of tokens.
    ///
    /// This respects the structural requirements of adding one element at a
    /// time, like you would get by calling [space()], [push()], or [line()].
    pub fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Item<L>>,
    {
        let mut it = it.into_iter();

        while let Some(item) = it.next() {
            self.push_item(item);
        }
    }

    /// Walk over all imports.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// let debug = rust::imported("std::fmt", "Debug");
    ///
    /// let ty = rust::imported("std::collections", "HashMap")
    ///     .with_arguments((rust::U32, debug.into_dyn()));
    ///
    /// let tokens = quote!(foo #ty baz);
    ///
    /// for import in tokens.walk_imports() {
    ///     println!("{:?}", import);
    /// }
    /// ```
    pub fn walk_imports(&self) -> WalkImports<'_, L> {
        WalkImports {
            queue: self.items.iter(),
        }
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
    /// assert_eq!("use byteorder::WriteBytesExt as _;\n", tokens.to_file_string().unwrap());
    /// ```
    ///
    /// [quote!]: crate::quote!
    pub fn register<T>(&mut self, tokens: T)
    where
        T: RegisterTokens<L>,
    {
        tokens.register_tokens(self);
    }

    /// Check if tokens contain no items.
    ///
    /// ```rust
    /// use genco::prelude::*;
    ///
    /// let tokens: Tokens<()> = quote!();
    ///
    /// assert!(tokens.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Add a single spacing to the token stream.
    pub fn space(&mut self) {
        // A space is already present.
        match self.items.last() {
            Some(Item::Space) => return,
            _ => (),
        }

        self.items.push(Item::Space);
    }

    /// Add a single push spacing operation.
    pub fn push(&mut self) {
        // Already a push or an empty line in the stream.
        // Another one will do nothing.
        match self.items.last() {
            Some(Item::Push) | Some(Item::Line) => return,
            _ => (),
        }

        self.items.push(Item::Push);
    }

    /// Assert that there's the necessary items to create one empty line at
    /// the end of the stream.
    pub fn line(&mut self) {
        let mut it = self.items.iter().rev();

        let last = it.next();
        let ntl = it.next();

        match (ntl, last) {
            // A push + line is already at the end of the stream.
            (Some(Item::Push), Some(Item::Line)) => (),
            (_, Some(Item::Push)) => {
                self.items.push(Item::Line);
            }
            // Assert that there is something to push behind us.
            (_, Some(..)) => {
                self.items.push(Item::Push);
                self.items.push(Item::Line);
            }
            // do nothing.
            _ => (),
        }
    }

    /// Assert that there's the necessary items to create one empty line at
    /// the end of the stream.
    #[deprecated = "use `line` function instead"]
    pub fn push_line(&mut self) {
        self.line();
    }

    /// Add a single indentation to the token stream.
    pub fn indent(&mut self) {
        let n = match self.items.pop() {
            None => NonZeroI16::new(1),
            Some(Item::Indentation(level)) => NonZeroI16::new(level.get() + 1),
            Some(item) => {
                self.items.push(item);
                NonZeroI16::new(1)
            }
        };

        if let Some(n) = n {
            self.items.push(Item::Indentation(n));
        }
    }

    /// Add a single unindentation to the token stream.
    pub fn unindent(&mut self) {
        let n = match self.items.pop() {
            None => NonZeroI16::new(-1),
            Some(Item::Indentation(level)) => NonZeroI16::new(level.get() - 1),
            Some(item) => {
                self.items.push(item);
                NonZeroI16::new(-1)
            }
        };

        if let Some(n) = n {
            self.items.push(Item::Indentation(n));
        }
    }

    /// Format the tokens.
    pub fn format(&self, out: &mut Formatter, config: &mut L::Config, level: usize) -> fmt::Result {
        for element in &self.items {
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
            formatter.force_new_line()?;
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
            formatter.force_new_line()?;
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
            formatter.force_new_line()?;
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

impl<L> Clone for Tokens<L>
where
    L: Lang,
{
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
        }
    }
}

impl<L> cmp::PartialEq for Tokens<L>
where
    L: Lang,
{
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<'a, L> cmp::PartialEq<Vec<Item<L>>> for Tokens<L>
where
    L: Lang,
{
    fn eq(&self, other: &Vec<Item<L>>) -> bool {
        self.items == *other
    }
}

impl<'a, L> cmp::PartialEq<Tokens<L>> for Vec<Item<L>>
where
    L: Lang,
{
    fn eq(&self, other: &Tokens<L>) -> bool {
        *self == other.items
    }
}

impl<'a, L> cmp::PartialEq<[Item<L>]> for Tokens<L>
where
    L: Lang,
{
    fn eq(&self, other: &[Item<L>]) -> bool {
        &*self.items == other
    }
}

impl<'a, L> cmp::PartialEq<Tokens<L>> for [Item<L>]
where
    L: Lang,
{
    fn eq(&self, other: &Tokens<L>) -> bool {
        self == &*other.items
    }
}

impl<L> cmp::Eq for Tokens<L> where L: Lang {}

/// Iterator over [Tokens].
///
/// This is created using [Tokens::into_iter()].
pub struct IntoIter<L>
where
    L: Lang,
{
    iter: vec::IntoIter<Item<L>>,
}

impl<L> Iterator for IntoIter<L>
where
    L: Lang,
{
    type Item = Item<L>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<L> IntoIterator for Tokens<L>
where
    L: Lang,
{
    type Item = Item<L>;
    type IntoIter = IntoIter<L>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/// Iterator over [Tokens].
///
/// This is created using [Tokens::iter()].
pub struct Iter<'a, L>
where
    L: Lang,
{
    iter: slice::Iter<'a, Item<L>>,
}

impl<'a, L: 'a> Iterator for Iter<'a, L>
where
    L: Lang,
{
    type Item = &'a Item<L>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, L> IntoIterator for &'a Tokens<L>
where
    L: Lang,
{
    type Item = &'a Item<L>;
    type IntoIter = Iter<'a, L>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, L: 'a> FromIterator<&'a Item<L>> for Tokens<L>
where
    L: Lang,
{
    fn from_iter<I: IntoIterator<Item = &'a Item<L>>>(iter: I) -> Tokens<L> {
        let mut tokens = Tokens::new();
        tokens.extend(iter.into_iter().cloned());
        tokens
    }
}

impl<L> FromIterator<Item<L>> for Tokens<L>
where
    L: Lang,
{
    fn from_iter<I: IntoIterator<Item = Item<L>>>(iter: I) -> Tokens<L> {
        let mut tokens = Tokens::new();
        tokens.extend(iter.into_iter());
        tokens
    }
}

/// An iterator over language-specific imported items.
///
/// Constructed using the [Tokens::walk_imports] method.
pub struct WalkImports<'a, L>
where
    L: Lang,
{
    queue: std::slice::Iter<'a, Item<L>>,
}

impl<'a, L> Iterator for WalkImports<'a, L>
where
    L: Lang,
{
    type Item = &'a L::Import;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.queue.next() {
            let import = match next {
                Item::LangBox(item) => item.as_import(),
                Item::Registered(item) => item.as_import(),
                _ => continue,
            };

            if let Some(import) = import {
                return Some(import);
            }
        }

        None
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

    impl_lang_item! {
        impl FormatTokens<Lang> for Import;
        impl From<Import> for LangBox<Lang>;

        impl LangItem<Lang> for Import {
            fn format(&self, out: &mut Formatter, _: &mut (), _: usize) -> fmt::Result {
                use std::fmt::Write as _;
                write!(out, "{}", self.0)
            }

            fn as_import(&self) -> Option<&Self> {
                Some(self)
            }
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

        let output: Vec<_> = toks.walk_imports().cloned().collect();

        let expected = vec![Import(1), Import(2)];

        assert_eq!(expected, output);
    }
}
