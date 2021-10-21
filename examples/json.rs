use lasagna::*;

#[derive(Named, Token, Clone, Debug, PartialEq, Eq)]
pub enum JsonToken {
    #[token = "{"]
    OpenBrace,
    #[token = "}"]
    CloseBrace,
    #[token = "="]
    Equal,
    #[token = ","]
    Comma,
    #[token]
    LitStr(LitStr),
}

#[derive(Named, Clone, Debug, PartialEq, Eq)]
#[name = "<string>"]
pub struct LitStr(pub String);

impl Token for LitStr {
    fn lex(lexer: &mut impl Lexer<Output = char>) -> Result<Spanned<Self>, ParseError> {
        let mut span = lexer.span(0);

        lexer.expect('"')?;

        let mut string = String::new();

        loop {
            let next = lexer.next();

            if let Some(next_char) = next.value {
                if next_char == '"' {
                    span |= lexer.span(0);

                    break;
                } else {
                    string.push(next_char);
                }
            } else {
                return Err(ParseError::msg(next.span, "expected end to string"));
            }
        }

        Ok(Spanned::new(Self(string), span))
    }
}

#[derive(Parse, Debug)]
pub enum Value {
    String(LitStr),
    #[parse(peek = OpenBrace)]
    Table(Table),
}

#[derive(Parse, Debug)]
pub struct Statement {
    pub ident: LitStr,
    pub equal: Equal,
    pub value: Value,
}

#[derive(Parse, Debug)]
pub struct Table {
    pub open: OpenBrace,
    pub stmts: Punctuated<Statement, Comma, CloseBrace>,
    pub close: CloseBrace,
}

fn main() {
    let source = r#"{
    "foo" = {
       "bar" = "baz",
       "bar" = "baz"
    },
}"#;

    let mut parser = SkipWhitespace::new(CharsLexer::new(source.chars())).parse_as::<JsonToken>();

    let table = parser.parse::<Table>().unwrap();

    println!("{:#?}", table);
}
