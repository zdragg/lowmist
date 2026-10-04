pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, snafu::Snafu)]
pub enum Error {}
