mod convert;
mod moving;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    ConvertError(Box<str>),
    MoveError(Box<str>),
}
