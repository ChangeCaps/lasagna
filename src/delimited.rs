use crate::{Parse, ParseBuffer, Parser};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Delimited<L, T, R> {
    pub left: L,
    pub content: T,
    pub right: R,
}

impl<Token, L, T, R> Parse<Token> for Delimited<L, T, R>
where
    Token: Clone + PartialEq,
    L: Parse<Token>,
    T: Parse<Token>,
    R: Parse<Token>,
{
    fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
    where
        P: Parser<Token, Error> + ?Sized,
        Error: crate::ParseError<Token>,
    {
        let left = parser.parse()?;
        let mut content = ParseBuffer::new();
        let right;

        let mut count = 1;

        loop {
            if parser.try_parse::<L>().is_some() {
                count += 1;
            } else if let Some(r) = parser.try_parse::<R>() {
                count -= 1;

                if count == 0 {
                    right = r;

                    break;
                }
            }

            if let Some(tok) = parser.next() {
                content.push(tok);
            } else {
                return Err(Error::unexpected_eof());
            }
        }

        Ok(Self {
            left,
            content: content.parse()?,
            right,
        })
    }
}
