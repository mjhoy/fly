use crate::error::Error;
use std::{io::Read, path::Path};

#[derive(Debug)]
pub struct Migration {
    pub up_sql: String,
    pub down_sql: String,
    pub identifier: String,
}

impl Migration {
    /// Parse a `Migration` from a file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Migration, Error> {
        let identifier = path
            .as_ref()
            .file_name()
            .ok_or_else(|| Error::BadFilename {
                name: path.as_ref().to_string_lossy().to_string(),
                reason: "requires a file name".to_string(),
            })?
            .to_str()
            .ok_or_else(|| Error::BadFilename {
                name: path.as_ref().to_string_lossy().to_string(),
                reason: "not utf-8 encoded".to_string(),
            })?
            .to_string();
        let mut file = std::fs::File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut statements = contents.split('\n');
        let mut up = String::new();
        let mut down = String::new();
        for line in &mut statements {
            if line == "-- up" {
                break;
            }
        }
        for line in &mut statements {
            if line == "-- down" {
                break;
            }
            up.push_str(line);
            up.push('\n');
        }
        for line in &mut statements {
            down.push_str(line);
            down.push('\n');
        }
        Ok(Migration {
            up_sql: up,
            down_sql: down,
            identifier,
        })
    }
}
