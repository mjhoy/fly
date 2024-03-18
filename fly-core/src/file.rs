use crate::{error::Error, migration::Migration};
use std::io::Read;
use std::path::Path;

pub fn list(migrate_dir: impl AsRef<Path>) -> Result<Vec<Migration>, Error> {
    let paths = std::fs::read_dir(migrate_dir.as_ref())?;

    let mut migrations = Vec::new();
    for path in paths {
        let path = path?.path();
        migrations.push(parse_migration(path)?);
    }
    Ok(migrations)
}

fn parse_migration(path: impl AsRef<Path>) -> Result<Migration, Error> {
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
