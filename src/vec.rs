use std::ops::{Deref, DerefMut};

use crate::{Parse, ParseError, Parser, Spanned, Token};

/// Parsable [`Vec`] that is terminated by a token.
///
/// # Note
/// Term is not consumed.
#[derive(Clone, Debug, Default)]
pub struct VecTerminated<T, Term> {
    pub vec: Vec<Spanned<T>>,
    pub termination: Option<Spanned<Term>>,
}

impl<T, Term, Source> Parse for VecTerminated<T, Term>
where
    T: Parse<Source = Source>,
    Term: Token<Source>,
{
    type Source = Source;

    #[inline]
    fn parse(parser: &mut impl Parser<Source = Source>) -> Result<Spanned<Self>, ParseError> {
        let mut span = parser.span(0);
        let mut vec = Vec::new();

        let mut termination = None;

        while !parser.is_empty() {
            let mut fork = parser.fork();

            if let Ok(term) = fork.next::<Term>() {
                span |= term.span;

                termination = Some(term);

                break;
            }

            vec.push(parser.parse()?);
        }

        Ok(Spanned::new(Self { vec, termination }, span))
    }
}

impl<T, Term> Deref for VecTerminated<T, Term> {
    type Target = Vec<Spanned<T>>;

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
pub struct Punctuated<T, P, Term> {
    pub values: Vec<Spanned<T>>,
    pub punctuation: Vec<Spanned<P>>,
    pub termination: Option<Spanned<Term>>,
}

impl<T, P, Term, Source> Parse for Punctuated<T, P, Term>
where
    T: Parse<Source = Source>,
    P: Parse<Source = Source>,
    Term: Token<Source>,
{
    type Source = Source;

    #[inline]
    fn parse(parser: &mut impl Parser<Source = Source>) -> Result<Spanned<Self>, ParseError> {
        let mut span = parser.span(0);
        let mut values = Vec::new();
        let mut punctuation = Vec::new();

        let mut termination = None;

        while !parser.is_empty() {
            let mut fork = parser.fork();

            if let Ok(term) = fork.next::<Term>() {
                span |= term.span;

                termination = Some(term);

                break;
            }

            values.push(parser.parse()?);

            if parser.is_empty() {
                break;
            }

            let mut fork = parser.fork();

            if let Ok(term) = fork.next::<Term>() {
                span |= term.span;

                termination = Some(term);

                break;
            }

            punctuation.push(parser.parse()?);
        }

        Ok(Spanned::new(
            Self {
                values,
                punctuation,
                termination,
            },
            span,
        ))
    }
}

impl<T, P, Term> Deref for Punctuated<T, P, Term> {
    type Target = Vec<Spanned<T>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<T, P, Term> DerefMut for Punctuated<T, P, Term> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}
