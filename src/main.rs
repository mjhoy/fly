use clap::Parser;
use command::Command;
use config::Config;
use db::Db;
use error::Error;
use migration::Migration;
use std::{io::Write, path::Path, time::SystemTime};

mod command;
mod config;
mod db;
mod error;
mod migration;

static MIGRATION_TEMPLATE: &str = "-- up\n\n-- down\n";

fn main() {
    match run_fly() {
        Ok(_) => {}
        Err(e) => {
            e.print_and_abort();
        }
    }
}

fn run_fly() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let command = Command::parse();
    let config = Config::from_env()?;
    let mut db = Db::connect(&config)?;
    db.create_migrations_table()?;

    let applied_migrations = db.get_applied_migrations()?;
    if config.debug {
        println!("migrations in schema table:");
        println!("{:?}", applied_migrations);
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
                if !applied_migrations.contains(&migration.identifier.as_str().to_owned()) {
                    println!("applying {}", migration.identifier);
                    let (up, _) = migration.up_down()?;
                    if config.debug {
                        println!("{}", up);
                    }
                    db.apply_migration(&up, migration)?;
                    any_mgrations_run = true;
                }
            }
            if !any_mgrations_run {
                println!("database is up to date");
            }
        }
        Command::Down => {
            if applied_migrations.is_empty() {
                println!("no migrations to revert");
                return Ok(());
            }
            let candidate = applied_migrations.last().unwrap();
            for migration in migrations.iter().rev() {
                if migration.identifier == *candidate {
                    println!("reverting {}", migration.identifier);
                    let (_, down) = migration.up_down()?;
                    if config.debug {
                        println!("{}", down);
                    }
                    db.rollback_migration(&down, migration)?;
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
            for name in &applied_migrations {
                if !known_migrations.contains(name) {
                    all_migrations.push(name.clone());
                }
            }
            all_migrations.sort();
            for migration in all_migrations {
                if known_migrations.contains(&migration) {
                    if applied_migrations.contains(&migration) {
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

fn get_migrations(config: &Config) -> Result<Vec<Migration>, Error> {
    let mut migrations = Vec::new();

    let paths = std::fs::read_dir(&config.migrate_dir).map_err(|e| {
        Error::Standard(format!(
            "problem reading migration directory ({}): {}",
            &config.migrate_dir, e
        ))
    })?;

    for path in paths {
        let path = path?.path();
        let migration = path_to_migration(&path)?;
        migrations.push(migration);
    }

    Ok(migrations)
}

fn path_to_migration(path: &Path) -> Result<Migration, Error> {
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
