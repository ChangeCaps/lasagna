use crate::{Lexer, ParseError, Span, Spanned, Token};

pub trait Parse: Sized {
    type Source;

    fn parse(parser: &mut impl Parser<Source = Self::Source>) -> Result<Spanned<Self>, ParseError>;
}

pub trait Parser {
    type Source;

    fn span(&mut self, length: usize) -> Span;

    fn next<T: Token<Self::Source>>(&mut self) -> Result<Spanned<T>, ParseError>;

    #[inline]
    fn peek<T: Token<Self::Source>>(&mut self) -> Option<Spanned<T>>
    where
        Self: Sized,
    {
        let mut fork = self.fork();

        fork.next().ok()
    }

    fn is_empty(&mut self) -> bool;

    fn fork(&mut self) -> Self;

    #[inline]
    fn parse<T: Parse<Source = Self::Source>>(&mut self) -> Result<Spanned<T>, ParseError>
    where
        Self: Sized,
    {
        T::parse(self)
    }

    #[inline]
    fn try_parse<T: Parse<Source = Self::Source>>(&mut self) -> Option<Spanned<T>>
    where
        Self: Sized,
    {
        let mut fork = self.fork();

        if let Ok(t) = T::parse(&mut fork) {
            *self = fork;

            Some(t)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkipWhitespace<L> {
    lexer: L,
}

/// [`Parser`] that skips whitespace between tokens.
impl<L> SkipWhitespace<L> {
    #[inline]
    pub fn new(lexer: L) -> Self {
        Self { lexer }
    }

    #[inline]
    fn skip_whitespace(&mut self)
    where
        L: Lexer<Output = char>,
    {
        while self
            .lexer
            .peek()
            .value
            .map(|c| c.is_whitespace())
            .unwrap_or(false)
        {
            self.lexer.consume();
        }
    }
}

impl<L> Parser for SkipWhitespace<L>
where
    L: Lexer<Output = char>,
{
    type Source = char;

    #[inline]
    fn span(&mut self, length: usize) -> Span {
        self.skip_whitespace();

        self.lexer.span(length)
    }

    #[inline]
    fn next<T: Token<Self::Source>>(&mut self) -> Result<Spanned<T>, ParseError> {
        self.skip_whitespace();

        T::lex(&mut self.lexer)
    }

    #[inline]
    fn is_empty(&mut self) -> bool {
        self.skip_whitespace();

        self.lexer.is_empty()
    }

    #[inline]
    fn fork(&mut self) -> Self {
        Self::new(self.lexer.fork())
    }
}
