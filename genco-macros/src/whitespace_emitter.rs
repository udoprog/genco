use crate::{Cursor, ItemBuffer};
use proc_macro2::{LineColumn, TokenStream};

/// Struct to deal with emitting the necessary spacing.
pub(crate) struct WhitespaceEmitter<'a> {
    receiver: &'a syn::Expr,
    /// Use to modify the initial line/column in case something was processed
    /// before the input was handed off to the quote parser.
    ///
    /// See [QuoteInParser].
    span_start: Option<LineColumn>,
    /// Override the end span of the quote parser.
    ///
    /// This causes whitespace to be emitted at the tail of the expression,
    /// unless it specifically reached the end of the span.
    span_end: Option<LineColumn>,
    // Used to determine the indentation state of a token.
    last_column: usize,
    /// Currently stored cursor.
    cursor: Option<Cursor>,
}

impl<'a> WhitespaceEmitter<'a> {
    pub(crate) fn new(
        receiver: &'a syn::Expr,
        span_start: Option<LineColumn>,
        span_end: Option<LineColumn>,
        last_column: usize,
    ) -> Self {
        Self {
            receiver,
            span_start,
            span_end,
            last_column,
            cursor: None,
        }
    }

    pub(crate) fn set_current(&mut self, cursor: Cursor) {
        self.cursor = Some(cursor);
    }

    // Reset cursor, so that registers don't count as items to be offset from.
    // This allows imports to be grouped without affecting formatting.
    pub(crate) fn reset(&mut self) {
        self.cursor = None;
    }

    pub(crate) fn step(
        &mut self,
        output: &mut TokenStream,
        item_buffer: &mut ItemBuffer,
        next: Cursor,
    ) {
        let mut next_start = next.start;

        // So we encountered the first ever token, while we have a spanned
        // start like `quote_in! { out => foo }`, `foo` is now `next`.
        //
        // What we want to do is treat the beginning out `out` as the
        // indentation position, so we adjust the token.
        //
        // But we also want to avoid situations like this:
        //
        // ```
        // quote_in! { out =>
        //     foo
        //     bar
        // }
        // ```
        //
        // If we would treat `out` as the start, `foo` would be seen as
        // unindented. So check if the first encountered token is on the
        // same line as the binding `out` or not before adjusting them!
        if let Some(span_start) = self.span_start.take() {
            if next_start.line == span_start.line {
                self.last_column = span_start.column;
                next_start = span_start;
            }
        }

        // Insert spacing if appropriate.
        self.handle_spacing(next_start, output, item_buffer);

        // Assign the current cursor to the next item.
        // This will then be used to make future indentation decisions.
        self.cursor = Some(next);
    }

    pub(crate) fn end(&mut self, output: &mut TokenStream, item_buffer: &mut ItemBuffer) {
        // evaluate whitespace in case we have an explicit end span.
        let end = match self.span_end.take() {
            Some(end) => end,
            None => return,
        };

        if let Some(span_start) = self.span_start.take() {
            if end.line == span_start.line {
                self.last_column = span_start.column;
            }
        }

        // Insert spacing if appropriate.
        self.handle_spacing(end, output, item_buffer);
    }

    /// Insert indentation and spacing if appropriate in the output token stream.
    fn handle_spacing(
        &mut self,
        next: LineColumn,
        output: &mut TokenStream,
        item_buffer: &mut ItemBuffer,
    ) {
        // Do nothing unless we have a cursor.
        let cursor = match &self.cursor {
            Some(cursor) => cursor,
            None => return,
        };

        let receiver = self.receiver;

        // Insert spacing if we are on the same line, but column has changed.
        if cursor.end.line == next.line {
            // Same line, but next item doesn't match.
            if cursor.end.column < next.column && self.last_column != next.column {
                item_buffer.flush(output);
                output.extend(quote::quote!(#receiver.space();));
            }

            return;
        }

        // Line changed. Determine whether to indent, unindent, or hard break the
        // line.
        item_buffer.flush(output);

        debug_assert!(next.line > cursor.start.line);

        let line_spaced = if next.line - cursor.end.line > 1 {
            output.extend(quote::quote!(#receiver.line();));
            true
        } else {
            false
        };

        if self.last_column < next.column {
            output.extend(quote::quote!(#receiver.indent();));
        } else if self.last_column > next.column {
            output.extend(quote::quote!(#receiver.unindent();));
        } else if !line_spaced {
            output.extend(quote::quote!(#receiver.push();));
        }

        self.last_column = next.column;
    }
}
