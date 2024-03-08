use crate::error::Error;
use std::{
    collections::HashMap,
    env::{self, VarError},
};

#[derive(Debug)]
pub struct Config {
    pub migrate_dir: String,
    pub connection_string: String,
    pub debug: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        let env_vars = env::vars().collect::<HashMap<String, String>>();
        let migrate_dir = get_env("MIGRATE_DIR", &env_vars)?;
        let debug = env_vars.get("DEBUG").unwrap_or(&"false".to_owned()) == "true";
        let connection_string = connection_string_from_env(&env_vars)?;

        Ok(Config {
            migrate_dir,
            connection_string,
            debug,
        })
    }
}

fn get_env(key: &str, vars: &HashMap<String, String>) -> Result<String, Error> {
    vars.get(key)
        .map(|s| s.to_owned())
        .ok_or(Error::Env((key.to_owned(), VarError::NotPresent)))
}

fn connection_string_from_env(env_vars: &HashMap<String, String>) -> Result<String, Error> {
    if let Ok(connection_string) = get_env("PG_CONNECTION_STRING", &env_vars) {
        Ok(connection_string)
    } else {
        let pg_user = get_env("PG_USER", &env_vars)?;
        let maybe_pg_password = env_vars.get("PG_PASSWORD").map(|s| s.to_owned());
        let pg_host = get_env("PG_HOST", &env_vars)?;
        let pg_port_str = get_env("PG_PORT", &env_vars)?;
        let pg_port = pg_port_str
            .parse::<u16>()
            .map_err(|_| Error::Standard("couldn't parse PG_PORT".to_owned()))?;
        let pg_db = get_env("PG_DB", &env_vars)?;

        let connection_string = if let Some(pg_password) = maybe_pg_password {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                pg_user, pg_password, pg_host, pg_port, pg_db
            )
        } else {
            format!("postgresql://{}@{}:{}/{}", pg_user, pg_host, pg_port, pg_db)
        };

        Ok(connection_string)
    }
}
