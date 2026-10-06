use proc_macro2::Span;

/// Internal line-column abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LineColumn {
    /// The line.
    pub(crate) line: usize,
    /// The column.
    pub(crate) column: usize,
}

impl LineColumn {
    /// The start of the given span.
    pub(crate) fn start(span: Span) -> Self {
        let span = span.unwrap().start();

        Self {
            line: span.line(),
            column: span.column(),
        }
    }

    /// The end of the given span.
    pub(crate) fn end(span: Span) -> Self {
        let span = span.unwrap().end();

        Self {
            line: span.line(),
            column: span.column(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor {
    // Span to use for diagnostics associated with the cursor.
    pub(crate) span: Span,
    // The start of the cursor.
    pub(crate) start: LineColumn,
    // The end of the cursor.
    pub(crate) end: LineColumn,
}

impl Cursor {
    /// Construct a cursor covering the given span.
    pub(crate) fn from_span(span: Span) -> Cursor {
        Self {
            span,
            start: LineColumn::start(span),
            end: LineColumn::end(span),
        }
    }

    /// Construct a cursor from the start of `a` to the end of `b`.
    pub(crate) fn join(a: Span, b: Span) -> Cursor {
        Self {
            span: a.join(b).unwrap_or(a),
            start: LineColumn::start(a),
            end: LineColumn::end(b),
        }
    }

    /// Calculate the start character for the cursor.
    pub(crate) fn first_character(self) -> Self {
        Cursor {
            span: self.span,
            start: self.start,
            end: LineColumn {
                line: self.start.line,
                column: self.start.column + 1,
            },
        }
    }

    /// Calculate the end character for the cursor.
    pub(crate) fn last_character(self) -> Self {
        Cursor {
            span: self.span,
            start: LineColumn {
                line: self.end.line,
                column: self.end.column.saturating_sub(1),
            },
            end: self.end,
        }
    }
}
