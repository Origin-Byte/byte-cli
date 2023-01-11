use thiserror::Error;

#[derive(Error, Debug)]
pub enum GutenError {
    #[error("Parsing error has occured")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("An IO error has occured")]
    IoError(#[from] std::io::Error),
    #[error("A tag provided is not supported")]
    UnsupportedTag,
}
