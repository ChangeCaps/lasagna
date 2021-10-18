use crate::{DynParser, MutOrOwned, ParseError, Parser, TokenParser, WhitespaceParser};

pub trait ParserExt<Token, Error>
where
    Error: ParseError<Token>,
    Self: Parser<Token, Error>,
{
    fn parse_as<'a, Out>(self) -> TokenParser<'a, Out, dyn DynParser<Token, Error> + 'a>
    where
        Self: 'a;

    fn pad_whitespace<'a, Out>(
        self,
    ) -> WhitespaceParser<Out, Box<dyn DynParser<Token, Error> + 'a>>
    where
        Self: 'a;
}

impl<Token, Error, P> ParserExt<Token, Error> for P
where
    Error: ParseError<Token>,
    Self: Parser<Token, Error>,
{
    fn parse_as<'a, Out>(self) -> TokenParser<'a, Out, dyn DynParser<Token, Error> + 'a>
    where
        Self: 'a,
    {
        TokenParser {
            parser: MutOrOwned::Owned(Box::new(self)),
            peek: None,
        }
    }

    fn pad_whitespace<'a, Out>(self) -> WhitespaceParser<Out, Box<dyn DynParser<Token, Error> + 'a>>
    where
        Self: 'a,
    {
        WhitespaceParser::new(Box::new(self))
    }
}
