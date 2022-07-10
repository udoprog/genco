use proc_macro2::Span;

use crate::fake::LineColumn;

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
    /// Construt a cursor.
    pub(crate) fn new(span: Span, start: LineColumn, end: LineColumn) -> Cursor {
        Self { span, start, end }
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
