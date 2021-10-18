use std::{iter::Peekable, str::Chars};

use crate::{ParseError, Parser};

#[derive(Clone, Debug)]
pub struct CharsParser<'a> {
    skip_whitespace: bool,
    chars: Peekable<Chars<'a>>,
}

impl<'a> CharsParser<'a> {
    #[inline]
    pub fn new(chars: impl Into<Chars<'a>>) -> Self {
        Self {
            skip_whitespace: false,
            chars: chars.into().peekable(),
        }
    }

    #[inline]
    pub fn skip_whitespace(mut self, skip: bool) -> Self {
        self.skip_whitespace = skip;
        self
    }
}

impl<'a, Error> Parser<char, Error> for CharsParser<'a>
where
    Error: ParseError<char>,
{
    fn next(&mut self) -> Option<char> {
        if self.skip_whitespace {
            while self.chars.peek()?.is_whitespace() {
                self.chars.next();
            }
        }

        self.chars.next()
    }

    fn peek(&mut self) -> Option<&char> {
        if self.skip_whitespace {
            while self.chars.peek()?.is_whitespace() {
                self.chars.next();
            }
        }

        self.chars.peek()
    }

    fn expect(&mut self, token: char) -> Result<(), Error> {
        let found = Parser::<char, Error>::next(self);

        if found == Some(token) {
            Ok(())
        } else {
            Err(Error::expected(found, &[token]))
        }
    }

    fn try_parse_with<P>(
        &mut self,
        parser: impl FnOnce(&mut dyn crate::DynParser<char, Error>) -> Result<P, Error>,
    ) -> Result<P, Error> {
        let mut fork = self.clone();

        match parser(&mut fork) {
            Ok(v) => {
                *self = fork;

                Ok(v)
            }
            Err(e) => Err(e),
        }
    }
}

impl<'a, T: Into<Chars<'a>>> From<T> for CharsParser<'a> {
    fn from(chars: T) -> Self {
        Self::new(chars)
    }
}
