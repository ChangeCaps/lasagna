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
pub struct LitStr {
    span: Span,
    pub string: String,
}

impl Spanned for LitStr {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl Token for LitStr {
    fn lex(lexer: &mut impl Lexer<Output = char>) -> Result<Self, Error> {
        let mut span = lexer.span(0);

        lexer.expect('"')?;

        let mut string = String::new();

        loop {
            let next = lexer.next();

            if let Some(next_char) = next {
                if next_char == '"' {
                    span |= lexer.span(0);

                    break;
                } else {
                    string.push(next_char);
                }
            } else {
                return Err(Error::spanned(span, "expected end to string"));
            }
        }

        Ok(Self {
            span: span | lexer.span(0),
            string,
        })
    }
}

#[derive(Spanned, Parse, Debug)]
pub struct Statement {
    pub ident: LitStr,
    pub equal: Equal,
}

fn main() {}
