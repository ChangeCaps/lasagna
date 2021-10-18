#[macro_export]
macro_rules! token {
    ($ident:ident) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $ident;
    };
}

#[macro_export]
macro_rules! parse {
    ($ty:ty > $expr:expr => $tgt:ty $(, $($tt:tt)*)?) => {
        impl $crate::Parse<$ty> for $tgt {
            #[inline]
            fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
            where
                P: $crate::Parser<$ty, Error> + ?Sized,
                Error: $crate::ParseError<$ty>,
            {
                parser.expect($expr)?;

                Ok(Self)
            }
        }

        $(parse!($($tt)*);)?
    };
    ($ty:ty > $pat:pat => $tgt:ty > $expr:expr $(, $($tt:tt)*)?) => {
        impl $crate::Parse<$ty> for $tgt {
            #[inline]
            fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
            where
                P: $crate::Parser<$ty, Error> + ?Sized,
                Error: $crate::ParseError<$ty>,
            {
                match parser.next()? {
                    Some($pat) => Ok($expr),
                    None => Err(Error::unexpected_eof()),
                    tok => Err(Error::expected(tok, $crate::TokenOrMessage::from_str(stringify!($tgt)))),
                }
            }
        }

        $(parse!($($tt)*);)?
    };
    ($chars:expr => $tgt:ty $(, $($tt:tt)*)?) => {
        impl $crate::Parse<char> for $tgt {
            #[inline]
            fn parse<Error, P>(parser: &mut P) -> Result<Self, Error>
            where
                P: $crate::Parser<char, Error> + ?Sized,
                Error: $crate::ParseError<char>,
            {
                for c in $chars.chars() {
                    parser.expect(c)?;
                }

                Ok(Self)
            }
        }

        $(parse!($($tt)*);)?
    };
    () => {};
}
