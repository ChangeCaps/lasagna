use std::ops::{Deref, DerefMut};

use crate::{Parse, ParseError, Parser, Span, Spanned, Token};

/// Parsable [`Vec`] that is terminated by a token.
///
/// # Note
/// Term is not consumed.
#[derive(Clone, Debug, Default)]
pub struct VecTerminated<T, Term> {
    span: Span,
    pub vec: Vec<T>,
    pub termination: Option<Term>,
}

impl<T, Term> Spanned for VecTerminated<T, Term> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<T, Term, Source> Parse for VecTerminated<T, Term>
where
    T: Parse<Source = Source>,
    Term: Token<Source>,
{
    type Source = Source;

    #[inline]
    fn parse(parser: &mut impl Parser<Source = Source>) -> Result<Self, ParseError> {
        let span = parser.span(0);
        let mut vec = Vec::new();

        let mut termination = None;

        while !parser.is_empty() {
            let mut fork = parser.fork();

            if let Ok(term) = fork.next::<Term>() {
                *parser = fork;
                termination = Some(term);

                break;
            }

            vec.push(parser.parse()?);
        }

        Ok(Self {
            span: span | parser.span(0),
            vec,
            termination,
        })
    }
}

impl<T, Term> Deref for VecTerminated<T, Term> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T, Term> DerefMut for VecTerminated<T, Term> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

#[derive(Clone, Debug, Default)]
pub struct Punctuated<T, P> {
    span: Span,
    pub values: Vec<T>,
    pub punctuation: Vec<P>,
}

impl<T, P> Spanned for Punctuated<T, P> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<T, P, Source> Parse for Punctuated<T, P>
where
    T: Parse<Source = Source>,
    P: Parse<Source = Source>,
{
    type Source = Source;

    #[inline]
    fn parse(parser: &mut impl Parser<Source = Source>) -> Result<Self, ParseError> {
        let span = parser.span(0);
        let mut values = Vec::new();
        let mut punctuation = Vec::new();

        if !parser.is_empty() {
            loop {
                values.push(parser.parse()?);

                if let Some(punct) = parser.try_parse() {
                    punctuation.push(punct);
                } else {
                    break;
                }
            }
        }

        Ok(Self {
            span: span | parser.span(0),
            values,
            punctuation,
        })
    }
}

impl<T, P> Deref for Punctuated<T, P> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<T, P> DerefMut for Punctuated<T, P> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}
