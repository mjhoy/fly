use clap::Parser;
use command::Command;
use config::Config;
use migration::Migration;
use postgres::{Client, NoTls};
use std::{io::Write, path::Path, time::SystemTime};

mod command;
mod config;
mod migration;

static CREATE_MIGRATIONS_TABLE: &str = r#"
  CREATE TABLE IF NOT EXISTS migrations (
      id SERIAL PRIMARY KEY,
      name TEXT NOT NULL UNIQUE,
      created_at TIMESTAMP NOT NULL DEFAULT NOW()
  );
"#;

static MIGRATION_TEMPLATE: &str = "-- up\n\n-- down\n";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let command = Command::parse();
    let config = Config::from_env();
    let mut client = Client::connect(&connection_string(&config), NoTls)?;
    client.batch_execute(CREATE_MIGRATIONS_TABLE)?;

    let res = client.query("SELECT name FROM migrations", &[]).unwrap();
    let names: Vec<String> = res.iter().map(|row| row.get(0)).collect();
    if config.debug {
        println!("migrations in schema table:");
        println!("{:?}", names);
    }

    let mut migrations = get_migrations(&config)?;
    migrations.sort_by(|a, b| a.identifier.cmp(&b.identifier));
    if config.debug {
        println!("migrations in migrations dir:");
        println!("{:?}", migrations);
    }

    match command {
        Command::Up => {
            let mut any_mgrations_run = false;
            for migration in &migrations {
                if !names.contains(&migration.identifier.as_str().to_owned()) {
                    println!("applying {}", migration.identifier);
                    let (up, _) = migration.up_down()?;
                    if config.debug {
                        println!("{}", up);
                    }
                    let mut transaction = client.transaction()?;
                    transaction.batch_execute(&up)?;
                    transaction.execute(
                        "INSERT INTO migrations (name) VALUES ($1)",
                        &[&migration.identifier],
                    )?;
                    transaction.commit()?;
                    any_mgrations_run = true;
                }
            }
            if !any_mgrations_run {
                println!("database is up to date");
            }
        }
        Command::Down => {
            if names.is_empty() {
                println!("no migrations to revert");
                return Ok(());
            }
            let candidate = names.last().unwrap();
            for migration in migrations.iter().rev() {
                if migration.identifier == *candidate {
                    println!("reverting {}", migration.identifier);
                    let (_, down) = migration.up_down()?;
                    if config.debug {
                        println!("{}", down);
                    }
                    let mut transaction = client.transaction()?;
                    transaction.batch_execute(&down)?;
                    transaction.execute(
                        "DELETE FROM migrations WHERE name = $1",
                        &[&migration.identifier],
                    )?;
                    transaction.commit()?;
                    break;
                }
            }
        }
        Command::Status => {
            let mut all_migrations = Vec::new();
            let known_migrations = migrations
                .iter()
                .map(|m| m.identifier.clone())
                .collect::<Vec<String>>();
            for migration in &known_migrations {
                all_migrations.push(migration.clone())
            }
            for name in &names {
                if !known_migrations.contains(name) {
                    all_migrations.push(name.clone());
                }
            }
            all_migrations.sort();
            for migration in all_migrations {
                if known_migrations.contains(&migration) {
                    if names.contains(&migration) {
                        println!("{} [applied]", migration);
                    } else {
                        println!("{} [pending]", migration);
                    }
                } else {
                    println!("{} ** NO FILE **", migration);
                }
            }
        }
        Command::New(new_args) => {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("time went backwards")
                .as_secs();
            let filename = format!("{}/{}-{}.sql", config.migrate_dir, timestamp, new_args.name);
            let mut file = std::fs::File::create(&filename)?;
            file.write_all(MIGRATION_TEMPLATE.as_bytes())?;
            println!("Created file {}", filename);
        }
    }

    Ok(())
}

fn connection_string(config: &Config) -> String {
    format!(
        "postgresql://{}@{}:{}/{}",
        config.pg_user, config.pg_host, config.pg_port, config.pg_db
    )
}

fn get_migrations(config: &Config) -> Result<Vec<Migration>, Box<dyn std::error::Error>> {
    let mut migrations = Vec::new();

    let paths = std::fs::read_dir(&config.migrate_dir)?;

    for path in paths {
        let path = path?.path();
        let migration = path_to_migration(&path)?;
        migrations.push(migration);
    }

    Ok(migrations)
}

fn path_to_migration(path: &Path) -> Result<Migration, Box<dyn std::error::Error>> {
    let filename = path
        .file_name()
        .map(|s| s.to_str().expect("path with invalid unicode"));
    if let Some(filename) = filename {
        Ok(Migration {
            path: String::from(path.to_str().unwrap()),
            identifier: String::from(filename),
        })
    } else {
        Err("no filename".into())
    }
}
