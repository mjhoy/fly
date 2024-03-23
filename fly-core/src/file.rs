use crate::{error::Error, error::Result, migration::Migration};
use std::io::Read;
use std::path::Path;

pub fn list(migrate_dir: impl AsRef<Path>) -> Result<Vec<Migration>> {
    let paths = std::fs::read_dir(migrate_dir.as_ref())?;

    let mut migrations = Vec::new();
    for path in paths {
        let path = path?.path();
        if valid_migration_file_path(&path) {
            migrations.push(parse_migration_from_file(path)?);
        }
    }
    Ok(migrations)
}

// Check that a path is a file, ends in .sql, and does not start with a dot.
fn valid_migration_file_path(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.is_file()
        && path.ends_with(".sql")
        && path
            .file_name()
            .map_or(false, |f| !f.to_string_lossy().starts_with("."))
}

fn parse_migration_from_file(path: impl AsRef<Path>) -> Result<Migration> {
    let name = path
        .as_ref()
        .file_name()
        .ok_or(Error::FilenameRequired)?
        .to_str()
        .ok_or(Error::FilenameBadEncoding)?
        .to_string();
    let file = std::fs::File::open(&path)?;
    parse_migration(name, file)
}

fn parse_migration(name: String, mut reader: impl Read) -> Result<Migration> {
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    let mut statements = contents.split('\n');
    let mut up = String::new();
    let mut down = String::new();
    let mut has_up = false;
    let mut has_down = false;
    for line in &mut statements {
        if line == "-- up" {
            if has_down {
                return Err(Error::MigrationFileFormatError {
                    reason: "up migration must come first".to_string(),
                    name,
                });
            } else {
                has_up = true;
            }
            break;
        }
    }
    for line in &mut statements {
        if line == "-- up" {
            return Err(Error::MigrationFileFormatError {
                reason: "only one up migration allowed".to_string(),
                name,
            });
        }
        if line == "-- down" {
            has_down = true;
            break;
        }
        up.push_str(line);
        up.push('\n');
    }
    for line in &mut statements {
        if line == "-- down" {
            return Err(Error::MigrationFileFormatError {
                reason: "only one down migration allowed".to_string(),
                name,
            });
        }
        down.push_str(line);
        down.push('\n');
    }

    if !(has_down && has_up) {
        return Err(Error::MigrationFileFormatError {
            reason: "both up and down migrations must be defined".to_string(),
            name,
        });
    }
    Ok(Migration {
        up_sql: up.trim().to_string(),
        down_sql: down.trim().to_string(),
        name,
    })
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_parse_migration_empty_string() -> Result<()> {
        let migration_str = "".to_string();
        let result = parse_migration("foo".to_string(), Cursor::new(migration_str));

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "bad migration file format in foo: both up and down migrations must be defined"
        );

        Ok(())
    }

    #[test]
    fn test_parse_migration_just_up() -> Result<()> {
        let migration_str = "
-- up
create table users (id int);
"
        .to_string();
        let result = parse_migration("foo".to_string(), Cursor::new(migration_str));

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "bad migration file format in foo: both up and down migrations must be defined"
        );

        Ok(())
    }

    #[test]
    fn test_parse_migration_just_down() -> Result<()> {
        let migration_str = "
-- down
drop table users;
"
        .to_string();
        let result = parse_migration("foo".to_string(), Cursor::new(migration_str));

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "bad migration file format in foo: both up and down migrations must be defined"
        );

        Ok(())
    }

    #[test]
    fn test_parse_migration_multiple_ups() -> Result<()> {
        let migration_str = "
-- up
create table users (id int);

-- up
alter table users add column is_active boolean default true;

-- down
drop table users;
"
        .to_string();
        let result = parse_migration("foo".to_string(), Cursor::new(migration_str));

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "bad migration file format in foo: only one up migration allowed"
        );

        Ok(())
    }

    #[test]
    fn test_parse_migration_multiple_downs() -> Result<()> {
        let migration_str = "
-- up
create table users (id int);

-- down
alter table users remove column id;

-- down
drop table users;
"
        .to_string();
        let result = parse_migration("foo".to_string(), Cursor::new(migration_str));

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "bad migration file format in foo: only one down migration allowed"
        );

        Ok(())
    }

    #[test]
    fn test_parse_normal_migration() -> Result<()> {
        let migration_str = "
-- up
create table users (
  id int
);

-- down
drop table users;
"
        .to_string();
        let result = parse_migration("foo".to_string(), Cursor::new(migration_str));

        assert!(result.is_ok());
        assert_eq!(
            result.ok().unwrap(),
            Migration {
                name: "foo".to_string(),
                up_sql: "create table users (
  id int
);"
                .to_string(),
                down_sql: "drop table users;".to_string(),
            }
        );

        Ok(())
    }
}
