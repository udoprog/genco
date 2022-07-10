use crate::fake::LineColumn;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor {
    pub(crate) start: LineColumn,
    pub(crate) end: LineColumn,
}

impl Cursor {
    /// Construt a cursor.
    pub(crate) fn new(start: LineColumn, end: LineColumn) -> Cursor {
        Self { start, end }
    }

    /// Calculate the start character for the cursor.
    pub(crate) fn first_character(self) -> Self {
        Cursor {
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
            start: LineColumn {
                line: self.end.line,
                column: self.end.column.saturating_sub(1),
            },
            end: self.end,
        }
    }
}
