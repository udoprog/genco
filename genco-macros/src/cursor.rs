use proc_macro2::{LineColumn, Span};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor {
    pub(crate) start: LineColumn,
    pub(crate) end: LineColumn,
}

impl Cursor {
    /// Join two spans.
    pub(crate) fn join(a: Span, b: Span) -> Self {
        Cursor {
            start: a.start(),
            end: b.end(),
        }
    }

    /// Check that the cursor is not a mock cursor.
    ///
    /// See: https://github.com/alexcrichton/proc-macro2/issues/237
    pub(crate) fn check_compat(&self) -> syn::Result<()> {
        if self.start.line == 0
            && self.start.column == 0
            && self.end.line == 0
            && self.end.column == 0
        {
            return Err(syn::Error::new(
                Span::call_site(),
                "Your compiler does not support spans which is required by genco, see: https://github.com/rust-lang/rust/issues/54725"
            ));
        }

        Ok(())
    }

    /// Calculate the start character for the span.
    pub(crate) fn first_character(self) -> Self {
        Cursor {
            start: self.start,
            end: LineColumn {
                line: self.start.line,
                column: self.start.column + 1,
            },
        }
    }

    /// Calculate the span for the end character.
    pub(crate) fn last_character(self) -> Self {
        Cursor {
            start: LineColumn {
                line: self.end.line,
                column: self.end.column.saturating_sub(1),
            },
            end: self.end,
        }
    }
}

impl From<Span> for Cursor {
    fn from(span: Span) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
        }
    }
}

impl<'a> From<&'a Span> for Cursor {
    fn from(span: &'a Span) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
        }
    }
}
