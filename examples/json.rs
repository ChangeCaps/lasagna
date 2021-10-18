use lasagna::*;

token!(OpenBrace);
token!(CloseBrace);
token!(Equal);
token!(Comma);

parse! {
    // symbols
    "{" => OpenBrace,
    Token > Token::OpenBrace(OpenBrace) => OpenBrace,
    "}" => CloseBrace,
    Token > Token::CloseBrace(CloseBrace) => CloseBrace,
    "=" => Equal,
    Token > Token::Equal(Equal) => Equal,
    "," => Comma,
    Token > Token::Comma(Comma) => Comma,

    // compounds
    Token > Token::Ident(ident) => Ident > ident,
    Token > Token::Integer(int) => Integer > int,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ident(pub String);

impl Parse<char> for Ident {
    fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
    where
        P: Parser<char, Error> + ?Sized,
        Error: ParseError<char>,
    {
        parser.expect('"')?;

        let mut ident = String::new();

        loop {
            if let Some(tok) = parser.next()? {
                if tok == '"' {
                    break Ok(Ident(ident));
                } else {
                    ident.push(tok);
                }
            } else {
                break Err(Error::unexpected_eof());
            }
        }
    }
}

#[derive(Parse, Clone, Debug, PartialEq, Eq)]
pub enum Token {
    OpenBrace(OpenBrace),
    CloseBrace(CloseBrace),
    Equal(Equal),
    Comma(Comma),
    Ident(Ident),
    Integer(Integer),
}

#[derive(Parse, Debug)]
#[parse(specific(char), specific(Token))]
pub enum Value {
    Integer(Integer),
    String(Ident),
    Table(Table),
}

#[derive(Parse, Debug)]
pub struct Statement {
    pub lhs: Ident,
    pub equal: Equal,
    pub rhs: Value,
}

pub type Table = Delimited<OpenBrace, Punctuated<Statement, Comma>, CloseBrace>;

fn main() -> Result<(), String> {
    let json = r#"{
    "foo" = 10,
    "baz" = "bar",
    "table" = {
        "foo" = 42,
        "bar" = 42069,
    },
}"#;

    let mut parser = CharsParser::new(json.chars()).pad_whitespace::<Token>();

    let value = Table::parse::<String, _>(&mut parser)?;

    println!("{:#?}", value);

    Ok(())
}
