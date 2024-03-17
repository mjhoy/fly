use crate::error::Error;
use crate::migration::Migration;
use crate::{config::Config, migration::MigrationWithMeta};
use postgres::{Client, NoTls};
use tracing::debug;

static CREATE_MIGRATIONS_TABLE: &str = r#"
  CREATE TABLE IF NOT EXISTS migrations (
      id SERIAL PRIMARY KEY,
      name TEXT NOT NULL UNIQUE,
      up_sql TEXT NOT NULL,
      down_sql TEXT NOT NULL,
      created_at TIMESTAMP NOT NULL DEFAULT NOW()
  );
"#;

pub struct Db {
    client: Client,
}

impl Db {
    pub fn connect(config: &Config) -> Result<Db, Error> {
        let client = Client::connect(&config.connection_string, NoTls)?;
        Ok(Db { client })
    }

    pub fn create_migrations_table(&mut self) -> Result<(), Error> {
        self.client.batch_execute(CREATE_MIGRATIONS_TABLE)?;
        Ok(())
    }

    pub fn list(&mut self) -> Result<Vec<MigrationWithMeta>, Error> {
        let rows = self.client.query("SELECT * FROM migrations", &[])?;
        let migrations = rows
            .iter()
            .map(MigrationWithMeta::try_from)
            .collect::<Result<_, Error>>()?;
        Ok(migrations)
    }

    /// Panics if the INSERT statement does not return 1 row.
    pub fn run(&mut self, migration: &Migration) -> Result<MigrationWithMeta, Error> {
        debug!("inserting migration {:?}", migration);
        let mut transaction = self.client.transaction()?;
        transaction.batch_execute(&migration.up_sql)?;
        let rows = transaction.query(
            "INSERT INTO migrations (name, up_sql, down_sql) VALUES ($1, $2, $3) RETURNING *",
            &[&migration.name, &migration.up_sql, &migration.down_sql],
        )?;
        let [ref row] = rows[..] else {
            panic!("postgres inserted {} elements, expected 1", rows.len());
        };
        let migration = MigrationWithMeta::try_from(row)?;

        transaction.commit()?;
        Ok(migration)
    }

    pub fn rollback_migration(&mut self, migration: &Migration) -> Result<(), Error> {
        debug!("rolling back migration {:?}", migration);
        let mut transaction = self.client.transaction()?;
        transaction.batch_execute(&migration.down_sql)?;
        transaction.execute("DELETE FROM migrations WHERE name = $1", &[&migration.name])?;
        transaction.commit()?;
        Ok(())
    }
}
