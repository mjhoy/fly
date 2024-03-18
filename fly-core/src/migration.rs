use crate::error::Error;
use std::{cmp::Ordering, io::Read, path::Path, time::SystemTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub up_sql: String,
    pub down_sql: String,
    pub name: String,
}

impl PartialOrd for Migration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for Migration {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MigrationMeta {
    pub id: i32,
    pub created_at: SystemTime,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MigrationWithMeta {
    pub migration: Migration,
    pub meta: MigrationMeta,
}

impl Migration {
    /// Parse a `Migration` from a file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Migration, Error> {
        let name = path
            .as_ref()
            .file_name()
            .ok_or(Error::FilenameRequired)?
            .to_str()
            .ok_or(Error::FilenameBadEncoding)?
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
            name,
        })
    }
}
