use proc_macro2::{Span, TokenStream};
use syn::{Expr, LitStr};

pub(crate) struct ItemBuffer<'a> {
    receiver: &'a Expr,
    buffer: String,
}

impl<'a> ItemBuffer<'a> {
    /// Construct a new line buffer.
    pub(crate) fn new(receiver: &'a Expr) -> Self {
        Self {
            receiver,
            buffer: String::new(),
        }
    }

    /// Push the given character to the line buffer.
    pub(crate) fn push(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Push the given string to the line buffer.
    pub(crate) fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    /// Flush the line buffer if necessary.
    pub(crate) fn flush(&mut self, tokens: &mut TokenStream) {
        if !self.buffer.is_empty() {
            let receiver = self.receiver;
            let s = LitStr::new(&self.buffer, Span::call_site());
            tokens.extend(quote::quote!(#receiver.append(genco::ItemStr::Static(#s));));
            self.buffer.clear();
        }
    }
}
