use std::{
    cmp::Ordering,
    ops::{BitOr, BitOrAssign, Deref, DerefMut},
};

use crate::{Parse, ParseError, Parser};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpannedOption<T> {
    span: Span,
    value: Option<T>,
}

impl<T> Spanned for SpannedOption<T> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<T> Parse for SpannedOption<T>
where
    T: Parse,
{
    type Source = T::Source;

    #[inline]
    fn parse(parser: &mut impl Parser<Source = Self::Source>) -> Result<Self, ParseError> {
        let span = parser.span(0);

        if let Ok(t) = parser.parse::<T>() {
            Ok(Self {
                span: span | parser.span(0),
                value: Some(t),
            })
        } else {
            Ok(Self {
                span: span | parser.span(0),
                value: None,
            })
        }
    }
}

impl<T> Deref for SpannedOption<T> {
    type Target = Option<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for SpannedOption<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
