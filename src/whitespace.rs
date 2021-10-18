use crate::{Parse, ParseError, Parser, Punctuated};

crate::token!(Whitespace);

impl Parse<char> for Whitespace {
    #[inline]
    fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
    where
        P: Parser<char, Error> + ?Sized,
        Error: ParseError<char>,
    {
        while let Some(tok) = parser.peek() {
            if tok.is_whitespace() {
                parser.consume()?;
            } else {
                break;
            }
        }

        Ok(Self)
    }
}

pub type SeparateWhitespace<T> = Punctuated<T, Whitespace>;
