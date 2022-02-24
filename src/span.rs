use std::{
    ops::{BitOr, BitOrAssign, Deref, DerefMut},
    path::Path,
};

use crate::{string_allocator::static_str, Error, Parse, ParseStart, Parser};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourcePath {
    Path(&'static Path),
    Generated,
    Other(&'static str),
}

impl SourcePath {
    pub fn path(path: &Path) -> Self {
        let string = static_str(path.as_os_str().to_str().unwrap());
        Self::Path(Path::new(string))
    }
}

/// A struct that denotes a location in the source of a file.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    /// Path to source.
    pub path: SourcePath,
    /// Source.
    pub source: &'static str,
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
        assert_eq!(self.path, rhs.path);
        assert_eq!(self.source as *const _, rhs.source as *const _);

        let lhs_end = self.offset + self.length;
        let rhs_end = rhs.offset + rhs.length;
        let offset = self.offset.min(rhs.offset);
        let end = lhs_end.max(rhs_end);

        let (line, column) = if self.offset > rhs.offset {
            (rhs.line, rhs.column)
        } else {
            (self.line, self.column)
        };

        Self {
            path: self.path,
            source: self.source,
            line,
            column,
            offset,
            length: end - offset,
        }
    }
}

impl BitOrAssign for Span {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span").finish()
    }
}

impl std::fmt::Display for Span {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {} column: {}", self.line, self.column)
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for Span {
    #[inline]
    fn span(&self) -> Span {
        *self
    }
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
    type Token = T::Token;

    const START: ParseStart<Self::Token> = T::START;

    #[inline]
    fn parse(parser: &mut impl Parser<Self::Token>) -> Result<Self, Error> {
        let span = parser.span(0);

        if let Some(t) = parser.try_parse::<T>()? {
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
