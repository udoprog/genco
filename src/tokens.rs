//! A set of tokens that make up a single source-file.
//!
//! ## Example
//!
//! ```rust
//! use genco::{Tokens, Java};
//! let mut toks: Tokens<Java> = Tokens::new();
//! toks.append("foo");
//! ```

use super::formatter::Formatter;
use super::element::Element;
use super::write_tokens::WriteTokens;
use std::collections::LinkedList;
use super::custom::Custom;
use std::fmt;
use std::result;
use std::slice;
use super::contained::Contained::{self, Owned, Borrowed};

/// A set of tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokens<'element, C: 'element> {
    elements: Vec<Contained<'element, Element<'element, C>>>,
}

/// Generic methods.
impl<'element, C: 'element> Tokens<'element, C> {
    /// Create a new set of tokens.
    pub fn new() -> Tokens<'element, C> {
        Tokens { elements: Vec::new() }
    }

    /// Push a nested definition.
    pub fn nested<T>(&mut self, tokens: T)
    where
        T: Into<Tokens<'element, C>>,
    {
        self.elements.push(
            Owned(Element::Nested(Owned(tokens.into()))),
        );
    }

    /// Push a nested reference to a definition.
    pub fn nested_ref(&mut self, tokens: &'element Tokens<'element, C>) {
        self.elements.push(Owned(Element::Nested(Borrowed(tokens))));
    }

    /// Push a definition, guaranteed to be preceded with one newline.
    pub fn push<T>(&mut self, tokens: T)
    where
        T: Into<Tokens<'element, C>>,
    {
        self.elements.push(
            Owned(Element::Push(Owned(tokens.into()))),
        );
    }

    /// Push a reference to a definition.
    pub fn push_ref(&mut self, tokens: &'element Tokens<'element, C>) {
        self.elements.push(Owned(Element::Push(Borrowed(tokens))));
    }

    /// Append the given element.
    pub fn append<E>(&mut self, element: E)
    where
        E: Into<Element<'element, C>>,
    {
        self.elements.push(Owned(element.into()));
    }

    /// Append a reference to a definition.
    pub fn append_ref(&mut self, element: &'element Element<'element, C>) {
        self.elements.push(Borrowed(element));
    }

    /// Extend with another set of tokens.
    pub fn extend<I>(&mut self, it: I)
    where
        I: IntoIterator<Item = Element<'element, C>>,
    {
        self.elements.extend(it.into_iter().map(Owned));
    }

    /// Walk over all elements.
    pub fn walk_custom(&'element self) -> WalkCustomIter<'element, C> {
        let mut queue = LinkedList::new();
        queue.push_back(self);

        WalkCustomIter {
            queue: queue,
            current: None,
        }
    }
}

impl<'element, C: Custom> Tokens<'element, C> {
    /// Format the tokens.
    pub fn format(&self, out: &mut Formatter, extra: &mut C::Extra, level: usize) -> fmt::Result {
        for element in &self.elements {
            element.as_ref().format(out, extra, level)?;
        }

        Ok(())
    }

    /// Format token as file.
    pub fn to_file(self) -> result::Result<String, fmt::Error> {
        let mut output = String::new();
        let mut extra = C::Extra::default();
        output.write_file(self, &mut extra)?;
        Ok(output)
    }
}

impl<'element, C: Custom<Extra = ()>> Tokens<'element, C> {
    /// Format the tokens.
    pub fn to_string(self) -> result::Result<String, fmt::Error> {
        let mut output = String::new();
        output.write_tokens(self, &mut ())?;
        Ok(output)
    }
}

/// Methods only available for clonable elements.
impl<'element, C: Clone> Tokens<'element, C> {
    /// Join the set of tokens on the given element.
    pub fn join<E>(self, element: E) -> Tokens<'element, C>
    where
        E: Into<Element<'element, C>>,
    {
        let element = element.into();

        let len = self.elements.len();
        let mut it = self.elements.into_iter();

        let mut out: Vec<Contained<'element, Element<'element, C>>> =
            Vec::with_capacity(match len {
                v if v < 1 => v,
                v => v + v - 1,
            });

        if let Some(first) = it.next() {
            out.push(first);
        } else {
            return Tokens { elements: out };
        }

        while let Some(next) = it.next() {
            out.push(Owned(element.clone()));
            out.push(next);
        }

        Tokens { elements: out }
    }

    /// Join with spacing.
    pub fn join_spacing(self) -> Tokens<'element, C> {
        self.join(Element::Spacing)
    }

    /// Join with line spacing.
    pub fn join_line_spacing(self) -> Tokens<'element, C> {
        self.join(Element::LineSpacing)
    }
}

/// Convert custom elements.
impl<'element, C: Custom> From<C> for Tokens<'element, C> {
    fn from(value: C) -> Self {
        Tokens { elements: vec![Owned(Element::Custom(value))] }
    }
}

/// Convert strings.
impl<'element, C> From<&'element str> for Tokens<'element, C> {
    fn from(value: &'element str) -> Self {
        Tokens { elements: vec![Owned(Element::BorrowedLiteral(value))] }
    }
}

/// Convert element to tokens.
impl<'element, C> From<Element<'element, C>> for Tokens<'element, C> {
    fn from(value: Element<'element, C>) -> Self {
        Tokens { elements: vec![Owned(value)] }
    }
}

pub struct WalkCustomIter<'element, C: 'element> {
    queue: LinkedList<&'element Tokens<'element, C>>,
    current: Option<slice::Iter<'element, Contained<'element, Element<'element, C>>>>,
}

impl<'element, C: 'element> Iterator for WalkCustomIter<'element, C> {
    type Item = &'element C;

    fn next(&mut self) -> Option<Self::Item> {
        use self::Element::*;

        loop {
            match self.current {
                Some(ref mut current) => {
                    // read all nested tokens and queue them
                    // return any elements encountered.
                    while let Some(next) = current.next() {
                        match next.as_ref() {
                            &Append(ref tokens) |
                            &Push(ref tokens) |
                            &Nested(ref tokens) => {
                                self.queue.push_back(tokens.as_ref());
                                continue;
                            }
                            &Custom(ref custom) => return Some(custom),
                            _ => continue,
                        }
                    }

                    // fall through to set self.current to None
                }
                None => {
                    if let Some(next) = self.queue.pop_front() {
                        self.current = Some(next.elements.iter());
                    }

                    match self.current {
                        Some(_) => continue,
                        None => return None,
                    }
                }
            }

            self.current = None;
        }
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
