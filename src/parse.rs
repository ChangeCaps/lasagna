pub trait Parse<Source: ?Sized>: Sized {
    type Error;

    fn parse(source: Source) -> Result<Self, Self::Error>;
}
