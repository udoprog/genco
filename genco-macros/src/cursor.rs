use proc_macro2::Span;

use crate::fake::LineColumn;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor {
    pub(crate) start: LineColumn,
    pub(crate) end: LineColumn,
}

impl Cursor {
    /// Join two spans.
    pub(crate) fn join(a: Span, b: Span) -> syn::Result<Self> {
        Ok(Cursor {
            start: LineColumn::start(a)?,
            end: LineColumn::end(b)?,
        })
    }

    /// Construct a cursor from a span.
    pub(crate) fn from_span(span: Span) -> syn::Result<Self> {
        let (start, end) = LineColumn::pair(span)?;
        Ok(Self { start, end })
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
