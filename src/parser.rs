use crate::{Error, Lexer, Span, Token, TokenKind};

pub type ParseStart<T> = &'static StartTokens<'static, <T as Token>::Kind>;

#[derive(Clone, Copy, Debug)]
pub enum StartTokens<'a, T: TokenKind> {
    All,
    Any(&'a [&'a StartTokens<'a, T>]),
    One(&'a StartTokens<'a, T>),
    Token(&'a T),
}

impl<'a, T: TokenKind> StartTokens<'a, T> {
    pub fn contains(&self, kind: &T) -> bool {
        match self {
            Self::All => true,
            &Self::Any(kinds) => {
                for tok in kinds {
                    if tok.contains(kind) {
                        return true;
                    }
                }

                false
            }
            &Self::One(tok) => tok.contains(kind),
            &Self::Token(tok) => tok == kind,
        }
    }

    pub fn append_vec(&self, vec: &mut Vec<&'a T>) {
        match self {
            &Self::Any(kinds) => {
                for kind in kinds {
                    kind.append_vec(vec);
                }
            }
            &Self::One(kind) => kind.append_vec(vec),
            &Self::Token(kind) => {
                if !vec.contains(&kind) {
                    vec.push(kind);
                }
            }
            _ => {}
        }
    }

    pub fn to_vec(&self) -> Vec<&'a T> {
        let mut vec = Vec::new();
        self.append_vec(&mut vec);
        vec
    }
}

pub trait Parse: Sized {
    type Token: Token;

    const START: ParseStart<Self::Token>;

    fn parse(parser: &mut impl Parser<Self::Token>) -> Result<Self, Error>;

    #[allow(unused)]
    fn is_next(parser: &mut impl Parser<Self::Token>) -> Option<bool> {
        Some(Self::START.contains(&parser.peek().ok()??.kind()))
    }
}

pub trait Parser<T> {
    fn span(&mut self, length: usize) -> Span;

    fn next(&mut self) -> Result<T, Error>;

    fn peek(&mut self) -> Result<Option<&T>, Error>;

    fn is_empty(&mut self) -> bool;

    fn fork(&mut self) -> Self;

    fn parse<P: Parse<Token = T>>(&mut self) -> Result<P, Error>
    where
        Self: Sized,
    {
        P::parse(self)
    }

    fn try_parse<P: Parse<Token = T>>(&mut self) -> Result<Option<P>, Error>
    where
        Self: Sized,
    {
        match P::is_next(self) {
            Some(true) => Ok(Some(P::parse(self)?)),
            Some(false) => Ok(None),
            None => {
                let mut fork = self.fork();

                let parse = P::parse(&mut fork)?;

                *self = fork;

                Ok(Some(parse))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkipWhitespace<L, T> {
    lexer: L,
    peek: Option<T>,
}

/// [`Parser`] that skips whitespace between tokens.
impl<L, T> SkipWhitespace<L, T> {
    #[inline]
    pub fn new(lexer: L) -> Self {
        Self { lexer, peek: None }
    }

    #[inline]
    fn skip_whitespace(&mut self)
    where
        L: Lexer<Output = char>,
    {
        while self
            .lexer
            .peek()
            .map(|c| c.is_whitespace())
            .unwrap_or(false)
        {
            self.lexer.consume();
        }
    }
}

impl<L, T> Parser<T> for SkipWhitespace<L, T>
where
    L: Lexer<Output = char>,
    T: Token<char>,
{
    fn span(&mut self, length: usize) -> Span {
        self.skip_whitespace();

        self.lexer.span(length)
    }

    fn next(&mut self) -> Result<T, Error> {
        if let Some(peek) = self.peek.take() {
            Ok(peek)
        } else {
            T::lex(&mut self.lexer)
        }
    }

    fn peek(&mut self) -> Result<Option<&T>, Error> {
        if let Some(ref token) = self.peek {
            Ok(Some(token))
        } else {
            self.skip_whitespace();

            self.peek = Some(T::lex(&mut self.lexer)?);

            Ok(self.peek.as_ref())
        }
    }

    fn is_empty(&mut self) -> bool {
        self.skip_whitespace();

        self.lexer.is_empty()
    }

    fn fork(&mut self) -> Self {
        Self::new(self.lexer.fork())
    }
}
