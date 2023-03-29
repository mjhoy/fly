use std::{collections::HashMap, env};

#[derive(Debug)]
pub struct Config {
    pub migrate_dir: String,
    pub pg_user: String,
    pub pg_host: String,
    pub pg_port: u16,
    pub pg_db: String,
    pub debug: bool,
}

impl Config {
    pub fn from_env() -> Self {
        let env_vars = env::vars().collect::<HashMap<String, String>>();
        Config {
            migrate_dir: env_vars
                .get("MIGRATE_DIR")
                .expect("MIGRATE_DIR not set")
                .to_owned(),
            pg_user: env_vars.get("PG_USER").expect("PG_USER not set").to_owned(),
            pg_host: env_vars.get("PG_HOST").expect("PG_HOST not set").to_owned(),
            pg_port: env_vars
                .get("PG_PORT")
                .expect("PG_PORT not set")
                .parse()
                .unwrap(),
            pg_db: env_vars.get("PG_DB").expect("PG_DB not set").to_owned(),
            debug: env_vars.get("DEBUG").unwrap_or(&"false".to_owned()) == "true",
        }
    }
}
