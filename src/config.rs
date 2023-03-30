use crate::error::Error;
use std::{collections::HashMap, env};

#[derive(Debug)]
pub struct Config {
    pub migrate_dir: String,
    pub pg_user: String,
    pub pg_password: Option<String>,
    pub pg_host: String,
    pub pg_port: u16,
    pub pg_db: String,
    pub debug: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        let env_vars = env::vars().collect::<HashMap<String, String>>();
        let migrate_dir = get_env("MIGRATE_DIR")?;
        let pg_user = get_env("PG_USER")?;
        let pg_password = env_vars.get("PG_PASSWORD").map(|s| s.to_owned());
        let pg_host = get_env("PG_HOST")?;
        let pg_port_str = get_env("PG_PORT")?;
        let pg_port = pg_port_str
            .parse()
            .map_err(|_| Error::Standard("couldn't parse PG_PORT".to_owned()))?;
        let pg_db = get_env("PG_DB")?;
        let debug = env_vars.get("DEBUG").unwrap_or(&"false".to_owned()) == "true";
        Ok(Config {
            migrate_dir,
            pg_user,
            pg_password,
            pg_host,
            pg_port,
            pg_db,
            debug,
        })
    }
}

fn get_env(key: &str) -> Result<String, Error> {
    env::var(key).map_err(|e| Error::Env((key.to_owned(), e)))
}
