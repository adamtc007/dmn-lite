//! Identifier and source-location types shared across parser, compiler, and engine.

use std::fmt;

/// Byte-offset span into the original source text.
///
/// Every AST and IR node carries a span so that diagnostics can point at the
/// exact bytes that caused an error. Byte offsets (not char or line/column
/// indices) are used because they are encoding-deterministic and required for
/// V&S §11.5 source-normalisation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    /// Inclusive start byte offset.
    pub start: u32,
    /// Exclusive end byte offset.
    pub end: u32,
}

impl SourceSpan {
    /// Construct a span from start (inclusive) and end (exclusive) byte offsets.
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Length in bytes.
    pub fn len(self) -> u32 {
        self.end - self.start
    }

    /// True when the span covers zero bytes.
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Merge two spans into the smallest span that covers both.
    ///
    /// Used by parent AST nodes to derive their span from their children.
    pub fn merge(self, other: SourceSpan) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// Whether a numeric literal was written as an integer or decimal.
///
/// Stored on `NumberLitAst` at parse time so Phase 1.2 type-checking can
/// reject (for example) a decimal literal assigned to an `integer`-typed
/// output without re-scanning the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberKind {
    /// No decimal point: `42`, `-7`.
    Integer,
    /// Has a decimal point: `3.14`, `-0.5`.
    Decimal,
}
