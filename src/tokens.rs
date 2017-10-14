//! A set of tokens that make up a single source-file.
//!
//! ## Example
//!
//! ```rust
//! use genco::{Tokens, Java};
//! let mut toks: Tokens<Java> = Tokens::new();
//! toks.append("foo");
//! ```

use super::into_tokens::IntoTokens;
use super::formatter::Formatter;
use super::element::Element::{self, Push, Nested};
use super::write_tokens::WriteTokens;
use std::collections::LinkedList;
use super::custom::Custom;
use std::fmt;
use std::result;
use super::con::Con::{Owned, Borrowed};
use std::vec;
use std::iter::FromIterator;

/// A set of tokens.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tokens<'el, C: 'el> {
    elements: Vec<Element<'el, C>>,
}

/// Generic methods.
impl<'el, C: 'el> Tokens<'el, C> {
    /// Create a new set of tokens.
    pub fn new() -> Tokens<'el, C> {
        Tokens { elements: Vec::new() }
    }

    /// Push a nested definition.
    pub fn nested<T>(&mut self, tokens: T)
    where
        T: IntoTokens<'el, C>,
    {
        self.elements.push(Nested(Owned(tokens.into_tokens())));
    }

    /// Push a nested reference to a definition.
    pub fn nested_ref(&mut self, tokens: &'el Tokens<'el, C>) {
        self.elements.push(Nested(Borrowed(tokens)));
    }

    /// Push a definition, guaranteed to be preceded with one newline.
    pub fn push<T>(&mut self, tokens: T)
    where
        T: IntoTokens<'el, C>,
    {
        self.elements.push(Push(Owned(tokens.into_tokens())));
    }

    /// Push a reference to a definition.
    pub fn push_ref(&mut self, tokens: &'el Tokens<'el, C>) {
        self.elements.push(Push(Borrowed(tokens.into())));
    }

    /// Append the given element.
    pub fn append<E>(&mut self, element: E)
    where
        E: Into<Element<'el, C>>,
    {
        self.elements.push(element.into());
    }

    /// Append a reference to a definition.
    pub fn append_ref(&mut self, element: &'el Element<'el, C>) {
        self.elements.push(Element::Borrowed(element));
    }

    /// Extend with another set of tokens.
    pub fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Element<'el, C>>,
    {
        self.elements.extend(it.into_iter());
    }

    /// Walk over all elements.
    pub fn walk_custom(&self) -> WalkCustomIter<C> {
        let mut queue = LinkedList::new();
        queue.extend(self.elements.iter());
        WalkCustomIter { queue: queue }
    }

    /// Check if tokens contain no elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<'el, C> IntoIterator for Tokens<'el, C> {
    type Item = Element<'el, C>;
    type IntoIter = vec::IntoIter<Element<'el, C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'el, C: Custom> Tokens<'el, C> {
    /// Format the tokens.
    pub fn format(&self, out: &mut Formatter, extra: &mut C::Extra, level: usize) -> fmt::Result {
        for element in &self.elements {
            element.format(out, extra, level)?;
        }

        Ok(())
    }

    /// Format token as file.
    pub fn to_file(self) -> result::Result<String, fmt::Error> {
        self.to_file_with(C::Extra::default())
    }

    /// Format token as file with the given extra.
    pub fn to_file_with(self, mut extra: C::Extra) -> result::Result<String, fmt::Error> {
        let mut output = String::new();
        output.write_file(self, &mut extra)?;
        Ok(output)
    }

    /// Format the tokens.
    pub fn to_string(self) -> result::Result<String, fmt::Error> {
        self.to_string_with(C::Extra::default())
    }

    /// Format the tokens with the given extra.
    pub fn to_string_with(self, mut extra: C::Extra) -> result::Result<String, fmt::Error> {
        let mut output = String::new();
        output.write_tokens(self, &mut extra)?;
        Ok(output)
    }
}

/// Methods only available for clonable elements.
impl<'el, C: Clone> Tokens<'el, C> {
    /// Join the set of tokens on the given element.
    pub fn join<E>(self, element: E) -> Tokens<'el, C>
    where
        E: Into<Element<'el, C>>,
    {
        let element = element.into();

        let len = self.elements.len();
        let mut it = self.elements.into_iter();

        let mut out: Vec<Element<'el, C>> = Vec::with_capacity(match len {
            v if v < 1 => v,
            v => v + v - 1,
        });

        if let Some(first) = it.next() {
            out.push(first);
        } else {
            return Tokens { elements: out };
        }

        while let Some(next) = it.next() {
            out.push(element.clone());
            out.push(next);
        }

        Tokens { elements: out }
    }

    /// Join with spacing.
    pub fn join_spacing(self) -> Tokens<'el, C> {
        self.join(Element::Spacing)
    }

    /// Join with line spacing.
    pub fn join_line_spacing(self) -> Tokens<'el, C> {
        self.join(Element::LineSpacing)
    }
}

impl<'el, C> IntoTokens<'el, C> for Tokens<'el, C> {
    fn into_tokens(self) -> Tokens<'el, C> {
        self
    }
}

/// Convert collection to tokens.
impl<'el, C> IntoTokens<'el, C> for Vec<Tokens<'el, C>> {
    fn into_tokens(self) -> Tokens<'el, C> {
        Tokens { elements: self.into_iter().map(Into::into).collect() }
    }
}

into_tokens_impl_from_generic!(Vec<Tokens<'el, C>>);

/// Convert element to tokens.
impl<'el, C> IntoTokens<'el, C> for Element<'el, C> {
    fn into_tokens(self) -> Tokens<'el, C> {
        Tokens { elements: vec![self] }
    }
}

into_tokens_impl_from_generic!(Element<'el, C>);

/// Convert custom elements.
impl<'el, C> IntoTokens<'el, C> for C
where
    C: Custom,
{
    fn into_tokens(self) -> Tokens<'el, C> {
        Tokens { elements: vec![self.into()] }
    }
}

/// Convert custom elements.
impl<'el, C> IntoTokens<'el, C> for &'el C
where
    C: Custom,
{
    fn into_tokens(self) -> Tokens<'el, C> {
        Tokens { elements: vec![self.into()] }
    }
}

/// Convert borrowed strings.
impl<'el, C> IntoTokens<'el, C> for &'el str {
    fn into_tokens(self) -> Tokens<'el, C> {
        Tokens { elements: vec![self.into()] }
    }
}

into_tokens_impl_from_generic!(&'el str);

/// Convert strings.
impl<'el, C> IntoTokens<'el, C> for String {
    fn into_tokens(self) -> Tokens<'el, C> {
        Tokens { elements: vec![self.into()] }
    }
}

into_tokens_impl_from_generic!(String);

impl<'el, C> FromIterator<&'el Element<'el, C>> for Tokens<'el, C> {
    fn from_iter<I: IntoIterator<Item = &'el Element<'el, C>>>(iter: I) -> Tokens<'el, C> {
        Tokens { elements: iter.into_iter().map(|e| Element::Borrowed(e)).collect() }
    }
}

impl<'el, C> FromIterator<Element<'el, C>> for Tokens<'el, C> {
    fn from_iter<I: IntoIterator<Item = Element<'el, C>>>(iter: I) -> Tokens<'el, C> {
        Tokens { elements: iter.into_iter().collect() }
    }
}

pub struct WalkCustomIter<'el, C: 'el> {
    queue: LinkedList<&'el Element<'el, C>>,
}

impl<'el, C: 'el> Iterator for WalkCustomIter<'el, C> {
    type Item = &'el C;

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
                Push(ref tokens) |
                Nested(ref tokens) |
                Append(ref tokens) => {
                    self.queue.extend(tokens.as_ref().elements.iter());
                }
                Custom(ref custom) => return Some(custom.as_ref()),
                _ => {}
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::Tokens;
    use custom::Custom;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Lang(u32);

    impl Custom for Lang {
        type Extra = ();
    }

    #[test]
    fn test_join() {
        let s = String::from("foo");
        let mut toks: Tokens<()> = Tokens::new();

        // locally borrowed string
        toks.append(s.as_str());
        // static string
        toks.append("bar");
        // owned literal
        toks.append(String::from("nope"));

        let toks = toks.join_spacing();

        assert_eq!("foo bar nope", toks.to_string().unwrap().as_str());
    }

    #[test]
    fn test_walk_custom() {
        let mut toks: Tokens<Lang> = Tokens::new();

        toks.push(toks!("1:1", Lang(1), "1:2"));

        // static string
        toks.append("bar");

        toks.nested(toks!("2:1", "2:2", toks!("3:1", "3:2"), Lang(2)));

        // owned literal
        toks.append(String::from("nope"));

        let output: Vec<_> = toks.walk_custom().cloned().collect();

        let expected = vec![Lang(1), Lang(2)];

        assert_eq!(expected, output);
    }
}
