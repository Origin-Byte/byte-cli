use thiserror::Error;

#[derive(Error, Debug)]
pub enum GutenError {
    #[error("Launchpad must be initialized to initialize any slots")]
    SlotsMustInitializeLaunchpad,
    #[error("Parsing error has occured")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("An IO error has occured")]
    IoError(#[from] std::io::Error),
}
