use anyhow::{anyhow, Result};
use assert_cmd::assert::Assert;
use assert_cmd::prelude::*;
use predicates::prelude::predicate;
use std::io::Write;
use std::path::Path;
use std::{
    fs::{self, OpenOptions},
    process::Command,
};
use tempfile::tempdir;

mod common;

#[test]
fn test_unexpected_command() -> Result<()> {
    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("sup");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand 'sup'"));

    Ok(())
}

#[test]
fn test_unexpected_argument() -> Result<()> {
    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("up").arg("foo");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument 'foo'"));

    Ok(())
}

#[test]
fn test_requires_migrate_dir_set() -> Result<()> {
    let workdir = tempdir()?.into_path();

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("up");
    cmd.current_dir(&workdir);

    cmd.assert().failure().stderr(predicate::str::contains(
        "required environment variable MIGRATE_DIR not set",
    ));

    Ok(())
}

#[test]
fn test_requires_database_env_set() -> Result<()> {
    let workdir = tempdir()?.into_path();
    let env_file = workdir.join(".env");

    fs::write(
        &env_file,
        format!(
            r#"
MIGRATE_DIR={}
"#,
            env_file.to_string_lossy()
        ),
    )?;

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("up");
    cmd.current_dir(&workdir);

    cmd.assert().failure().stderr(predicate::str::contains(
        "required environment variable PG_USER not set",
    ));

    Ok(())
}

#[test]
fn test_returns_ok() -> Result<()> {
    let workdir = tempdir()?.into_path();
    let env_file = workdir.join(".env");
    let migrate_dir = workdir.join("migrations");
    fs::create_dir(&migrate_dir)?;
    let database = common::TestDatabase::new()?;

    fs::write(
        &env_file,
        format!(
            r#"
MIGRATE_DIR={migrate_dir}
PG_HOST={host}
PG_USER={user}
PG_PORT={port}
PG_DB={database}
"#,
            migrate_dir = migrate_dir.to_string_lossy(),
            host = database.host,
            user = database.user,
            port = database.port,
            database = database.database
        ),
    )?;

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("up");
    cmd.current_dir(&workdir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("database is up to date"));

    Ok(())
}

#[test]
fn test_accepts_pg_connection_string() -> Result<()> {
    let workdir = tempdir()?.into_path();
    let env_file = workdir.join(".env");
    let migrate_dir = workdir.join("migrations");
    fs::create_dir(&migrate_dir)?;
    let database = common::TestDatabase::new()?;

    fs::write(
        &env_file,
        format!(
            r#"
MIGRATE_DIR={migrate_dir}
PG_CONNECTION_STRING=postgres://{user}@{host}:{port}/{database}
"#,
            migrate_dir = migrate_dir.to_string_lossy(),
            host = database.host,
            user = database.user,
            port = database.port,
            database = database.database
        ),
    )?;

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("up");
    cmd.current_dir(&workdir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("database is up to date"));

    Ok(())
}

#[test]
fn test_migrates_up_and_down() -> Result<()> {
    let workdir = tempdir()?.into_path();
    let env_file = workdir.join(".env");
    let migrate_dir = workdir.join("migrations");
    fs::create_dir(&migrate_dir)?;
    let database = common::TestDatabase::new()?;

    fs::write(
        &env_file,
        format!(
            r#"
MIGRATE_DIR={migrate_dir}
PG_HOST={host}
PG_USER={user}
PG_PORT={port}
PG_DB={database}
"#,
            migrate_dir = migrate_dir.to_string_lossy(),
            host = database.host,
            user = database.user,
            port = database.port,
            database = database.database
        ),
    )?;

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("new");
    cmd.arg("create-users");
    cmd.current_dir(&workdir);

    let output = cmd.output()?;
    let assert = Assert::new(output.clone());
    assert
        .success()
        .stdout(predicate::str::contains("Created file"));

    let output = String::from_utf8(output.stdout)?;
    let (_, filename) = output
        .rsplit_once(' ')
        .ok_or(anyhow!("filename not found"))?;

    let filename = filename.trim();

    let mut f = OpenOptions::new()
        .write(true)
        .create_new(false)
        .open(&workdir.join(filename))?;

    write!(
        &mut f,
        "
-- up
create table users (id int);

-- down
drop table users;
",
    )
    .unwrap();
    drop(f);

    let migration_name = Path::new(filename).file_name().unwrap().to_string_lossy();

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("up");
    cmd.current_dir(&workdir);

    let output = cmd.output()?;
    let assert = Assert::new(output);
    assert.success().stdout(predicate::str::contains(format!(
        "applying {}",
        migration_name
    )));

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("status");
    cmd.current_dir(&workdir);
    let output = cmd.output()?;
    let assert = Assert::new(output);
    assert.success().stdout(predicate::str::contains(format!(
        "{} [applied]",
        migration_name
    )));

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("down");
    cmd.current_dir(&workdir);

    let output = cmd.output()?;
    let assert = Assert::new(output);
    assert.success().stdout(predicate::str::contains(format!(
        "reverting {}",
        migration_name
    )));

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("status");
    cmd.current_dir(&workdir);
    let output = cmd.output()?;
    let assert = Assert::new(output);
    assert.success().stdout(predicate::str::contains(format!(
        "{} [pending]",
        migration_name
    )));

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("up");
    cmd.current_dir(&workdir);
    cmd.output().unwrap();

    fs::remove_file(filename).unwrap();

    let mut cmd = Command::cargo_bin("fly")?;
    cmd.arg("status");
    cmd.current_dir(&workdir);
    let output = cmd.output()?;
    let assert = Assert::new(output);
    assert.success().stdout(predicate::str::contains(format!(
        "{} ** NO FILE",
        migration_name
    )));

    Ok(())
}
