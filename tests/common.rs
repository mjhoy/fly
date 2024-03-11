use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};

static DB_N: AtomicUsize = AtomicUsize::new(1);

pub struct TestDatabase {
    pub database: String,
    pub host: String,
    pub port: String,
    pub user: String,
}

impl TestDatabase {
    pub fn new() -> Result<TestDatabase> {
        let env_vars = env::vars().collect::<HashMap<String, String>>();
        let host = env_vars
            .get("TEST_PG_HOST")
            .context("must set TEST_PG_HOST")?
            .to_owned();
        let port = env_vars
            .get("TEST_PG_PORT")
            .context("must set TEST_PG_PORT")?
            .to_owned();
        let user = env_vars
            .get("TEST_PG_USER")
            .context("must set TEST_PG_USER")?
            .to_owned();

        let db_n = DB_N.fetch_add(1, Ordering::SeqCst);
        let name = format!("fly-test-{}", db_n);
        eprintln!("Creating database {}", &name);
        let mut command = Command::new("createdb");
        command.arg("-h");
        command.arg(&host);
        command.arg("-p");
        command.arg(&port);
        command.arg("-U");
        command.arg(&user);
        command.arg(&name);
        let result = command.output()?;
        assert!(result.status.success());
        Ok(TestDatabase {
            database: name,
            host,
            port,
            user,
        })
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        eprintln!("Dropping {}", &self.database);
        let mut command = Command::new("dropdb");
        command.arg(&self.database);
        if let Ok(result) = command.output() {
            if !result.status.success() {
                eprintln!("problem dropping database {}", self.database);
            }
        } else {
            eprintln!("problem dropping database {}", self.database);
        }
    }
}
