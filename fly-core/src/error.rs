use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Pg(#[from] postgres::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("required environment variable {name} not set")]
    MissingEnv { name: String },
    #[error("environment variable {name} could not be parsed")]
    BadEnvFormat { name: String },
    #[error("no filename given")]
    FilenameRequired,
    #[error("filename must be utf-8 encoded")]
    FilenameBadEncoding,
}
