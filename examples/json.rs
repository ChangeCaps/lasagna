use lasagna::Parse;

pub struct OpenBracket;

pub struct Integer {
    pub value: i64,
}

impl lasagna::Parse<String> for Integer {
    type Error = String;

    #[inline]
    fn parse(source: String) -> Result<Self, Self::Error> {
        let value = source.parse::<i64>().map_err(|err| err.to_string())?;

        Ok(Integer { value })
    }
}

pub struct Float {
    pub value: i64,
}

impl lasagna::Parse<String> for Integer {
    type Error = String;

    #[inline]
    fn parse(source: String) -> Result<Self, Self::Error> {
        let value = source.parse::<i64>().map_err(|err| err.to_string())?;

        Ok(Integer { value })
    }
}

#[derive(Parse)]
#[parse[String]]
pub enum Token {
    Integer(Integer),
}

fn main() {

}
