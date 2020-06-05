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

    /// Modify the start of the cursor.
    pub(crate) fn with_start(self, start: LineColumn) -> Self {
        Self { start, ..self }
    }

    /// Modify the end of the cursor.
    pub(crate) fn with_end(self, end: LineColumn) -> Self {
        Self { end, ..self }
    }

    /// Calculate the start character for the span.
    pub(crate) fn start_character(self) -> Self {
        Cursor {
            start: self.start,
            end: LineColumn {
                line: self.start.line,
                column: self.start.column + 1,
            },
        }
    }

    /// Calculate the end character for the span.
    pub(crate) fn end_character(self) -> Self {
        Cursor {
            start: LineColumn {
                line: self.end.line,
                column: self.end.column - 1,
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
