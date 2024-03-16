use anyhow::Result;
use assert_cmd::prelude::*;
use predicates::prelude::predicate;
use std::{fs, process::Command};
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
