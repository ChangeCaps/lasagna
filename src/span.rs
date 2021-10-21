use std::{
    cmp::Ordering,
    ops::{BitOr, BitOrAssign, Deref, DerefMut},
};

/// A struct that denotes a location in the source of a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    #[inline]
    pub fn new(value: T, span: Span) -> Self {
        Self { span, value }
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Spanned<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> std::fmt::Debug for Spanned<T>
where
    T: std::fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T> std::fmt::Display for Spanned<T>
where
    T: std::fmt::Display,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "'{}' at line: {}, column: {}",
            self.value, self.span.line, self.span.column
        )
    }
}
