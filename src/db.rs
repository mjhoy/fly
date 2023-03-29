use crate::config::Config;
use crate::migration::Migration;
use postgres::{Client, NoTls};

static CREATE_MIGRATIONS_TABLE: &str = r#"
  CREATE TABLE IF NOT EXISTS migrations (
      id SERIAL PRIMARY KEY,
      name TEXT NOT NULL UNIQUE,
      created_at TIMESTAMP NOT NULL DEFAULT NOW()
  );
"#;

pub struct Db {
    client: Client,
}

impl Db {
    pub fn connect(config: &Config) -> Result<Db, Box<dyn std::error::Error>> {
        let client = Client::connect(&connection_string(config), NoTls)?;
        Ok(Db { client })
    }

    pub fn create_migrations_table(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.client.batch_execute(CREATE_MIGRATIONS_TABLE)?;
        Ok(())
    }

    pub fn get_applied_migrations(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let rows = self.client.query("SELECT name FROM migrations", &[])?;
        let mut migrations = Vec::new();
        for row in rows {
            migrations.push(row.get(0));
        }
        Ok(migrations)
    }

    pub fn apply_migration(
        &mut self,
        sql: &str,
        migration: &Migration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut transaction = self.client.transaction()?;
        transaction.batch_execute(sql)?;
        transaction.execute(
            "INSERT INTO migrations (name) VALUES ($1)",
            &[&migration.identifier],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn rollback_migration(
        &mut self,
        sql: &str,
        migration: &Migration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut transaction = self.client.transaction()?;
        transaction.batch_execute(sql)?;
        transaction.execute(
            "DELETE FROM migrations WHERE name = $1",
            &[&migration.identifier],
        )?;
        transaction.commit()?;
        Ok(())
    }
}

fn connection_string(config: &Config) -> String {
    if let Some(password) = &config.pg_password {
        return format!(
            "postgresql://{}:{}@{}:{}/{}",
            config.pg_user, password, config.pg_host, config.pg_port, config.pg_db
        );
    } else {
        format!(
            "postgresql://{}@{}:{}/{}",
            config.pg_user, config.pg_host, config.pg_port, config.pg_db
        )
    }
}
