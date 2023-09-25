use proc_macro2::{Span, TokenStream};

use crate::Ctxt;

/// Buffer used to resolve static items.
pub(crate) struct StaticBuffer<'a> {
    cx: &'a Ctxt,
    buffer: String,
}

impl<'a> StaticBuffer<'a> {
    /// Construct a new line buffer.
    pub(crate) fn new(cx: &'a Ctxt) -> Self {
        Self {
            cx,
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
            let Ctxt { receiver, module } = self.cx;

            let s = syn::LitStr::new(&self.buffer, Span::call_site());
            tokens.extend(q::quote!(#receiver.append(#module::tokens::ItemStr::Static(#s));));
            self.buffer.clear();
        }
    }
}
