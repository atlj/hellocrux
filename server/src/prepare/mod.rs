mod convert;

pub type Result<T> = core::result::Result<T, Error>;

pub enum Error {
    ConvertError(Box<str>),
}
