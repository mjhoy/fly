pub enum Error {
    Standard(String),
    Pg(postgres::Error),
    Io(std::io::Error),
    Env((String, std::env::VarError)),
}

impl Error {
    pub fn print_and_abort(&self) {
        match self {
            Error::Standard(msg) => {
                eprintln!("[error] {}", msg);
                std::process::exit(1);
            }
            Error::Pg(e) => {
                eprintln!("[pg error] {}", e);
                std::process::exit(1);
            }
            Error::Io(e) => {
                eprintln!("[io error] {}", e);
                std::process::exit(1);
            }
            Error::Env(e) => {
                match e.1 {
                    std::env::VarError::NotPresent => {
                        eprintln!("[error] env var not set: {}", e.0);
                    }
                    std::env::VarError::NotUnicode(_) => {
                        eprintln!("[error] env var not unicode: {}", e.0);
                    }
                }
                std::process::exit(1);
            }
        }
    }
}

impl From<postgres::Error> for Error {
    fn from(err: postgres::Error) -> Self {
        Error::Pg(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Standard(err.to_string())
    }
}
