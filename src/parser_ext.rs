use std::fmt::Display;

use crate::{LexerParser, Named, Parse, Parser, ParserLexer};

pub trait ParserExt: Parser {
    #[inline]
    fn parse_as<T>(self) -> LexerParser<ParserLexer<T, Self>>
    where
        Self: Sized,
        T: Parse<Source = Self::Source> + PartialEq<T> + Named + Display + Clone,
    {
        LexerParser::new(ParserLexer::new(self))
    }
}

impl<T: Parser> ParserExt for T {}
