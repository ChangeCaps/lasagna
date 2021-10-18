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

        let mut count = 0;

        let right = loop {
            let _ = parser.try_parse_with(|parser| {
                if parser.parse::<L>().is_ok() {
                    count += 1;
                }

                Result::<(), _>::Err(Error::unexpected_eof())
            });

            {
                if count > 0 {
                    let _ = parser.try_parse_with(|parser| {
                        if parser.parse::<R>().is_ok() {
                            count -= 1;
                        }

                        Result::<(), _>::Err(Error::unexpected_eof())
                    });
                } else {
                    if let Some(right) = parser.try_parse() {
                        break right;
                    }
                }
            }

            if let Some(tok) = parser.next()? {
                content.push(tok);
            } else {
                return Err(Error::unexpected_eof());
            }
        };

        Ok(Self {
            left,
            content: content.parse()?,
            right,
        })
    }
}
