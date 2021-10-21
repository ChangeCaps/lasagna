use std::{
    cmp::Ordering,
    ops::{BitOr, BitOrAssign},
};

/// A struct that denotes a location in the source of a file.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    /// Line in source.
    pub line: usize,
    /// Column in source.
    pub column: usize,
    /// Character offset in source.
    pub offset: usize,
    /// Length in characters.
    pub length: usize,
}

impl BitOr for Span {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        let lhs_end = self.offset + self.length;
        let rhs_end = rhs.offset + rhs.length;
        let end = lhs_end.max(rhs_end);

        match self.offset.cmp(&rhs.offset) {
            Ordering::Less => Self {
                line: self.line,
                column: self.column,
                offset: self.offset,
                length: end - self.offset,
            },
            Ordering::Equal => Self {
                line: self.line,
                column: self.column,
                offset: self.offset,
                length: end - self.offset,
            },
            Ordering::Greater => Self {
                line: rhs.line,
                column: rhs.column,
                offset: rhs.offset,
                length: end - rhs.offset,
            },
        }
    }
}

impl BitOrAssign for Span {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}

impl<T: Spanned> Spanned for Box<T> {
    #[inline]
    fn span(&self) -> Span {
        self.as_ref().span()
    }
}
