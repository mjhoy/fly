use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Pg(#[from] postgres::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("required environment variable {name} not set")]
    MissingEnv { name: String },
    #[error("couldn't parse environment variable {name}")]
    BadEnvFormat { name: String },
    #[error("bad filename {name}: {reason}")]
    BadFilename { name: String, reason: String },
}
