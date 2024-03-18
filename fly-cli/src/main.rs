use anyhow::{Context, Result};
use clap::Parser;
use command::Command;
use fly::config::Config;
use fly::db::Db;
use std::{io::Write, time::SystemTime};
use tracing::{debug, info, Level};

mod command;

static MIGRATION_TEMPLATE: &str = "-- up\n\n-- down\n";

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let command = Command::parse();
    let config = Config::from_env()?;
    let level = if config.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .without_time()
        .with_target(false)
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("setting tracing subscriber failed")?;

    let mut db = Db::connect(&config).context("couldn't connect to database")?;
    db.create_migrations_table()
        .context("failed creating migrations table")?;

    let applied_migrations = db.list()?;

    debug!("migrations in schema table:");
    debug!(
        "{}",
        if applied_migrations.is_empty() {
            "(empty)".to_string()
        } else {
            applied_migrations
                .iter()
                .map(|migration| format!("{:?}", &migration))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    let mut migrations = fly::file::list(&config.migrate_dir)?;
    migrations.sort_by(|a, b| a.name.cmp(&b.name));

    debug!("migrations in migrations dir:");
    debug!(
        "{}",
        if migrations.is_empty() {
            "(empty)".to_string()
        } else {
            migrations
                .iter()
                .map(|m| m.name.clone())
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    match command {
        Command::Up => {
            let mut any_migrations_run = false;
            for migration in &migrations {
                if !applied_migrations
                    .iter()
                    .any(|m| m.migration.name == migration.name)
                {
                    info!("applying {}", migration.name);
                    debug!("{}", migration.up_sql);
                    db.run(migration)?;
                    any_migrations_run = true;
                }
            }
            if !any_migrations_run {
                info!("database is up to date");
            }
        }
        Command::Down => {
            if applied_migrations.is_empty() {
                info!("no migrations to revert");
                return Ok(());
            }
            let candidate = applied_migrations.last().unwrap();
            for migration in migrations.iter().rev() {
                if migration.name == *candidate.migration.name {
                    info!("reverting {}", migration.name);
                    debug!("{}", migration.down_sql);
                    db.rollback_migration(migration)?;
                    break;
                }
            }
        }
        Command::Status => {
            let mut all_migrations = Vec::new();
            let known_migrations = migrations;
            for migration in &known_migrations {
                all_migrations.push(migration.clone())
            }
            for migration in applied_migrations.iter().map(|m| &m.migration) {
                if !known_migrations.contains(migration) {
                    all_migrations.push(migration.clone());
                }
            }
            all_migrations.sort();
            for migration in all_migrations {
                if known_migrations.contains(&migration) {
                    if applied_migrations.iter().any(|m| m.migration == migration) {
                        info!("{} [applied]", migration.name);
                    } else {
                        info!("{} [pending]", migration.name);
                    }
                } else {
                    info!("{} ** NO FILE **", migration.name);
                }
            }
        }
        Command::New(new_args) => {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("time went backwards")
                .as_secs();
            let filename = format!("{}-{}.sql", timestamp, new_args.name);
            let path = config.migrate_dir.join(filename);
            let mut file = std::fs::File::create(&path)?;
            file.write_all(MIGRATION_TEMPLATE.as_bytes())?;
            info!("Created file {}", path.display());
        }
    }

    Ok(())
}
