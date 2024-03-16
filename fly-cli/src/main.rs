use anyhow::{Context, Result};
use clap::Parser;
use command::Command;
use fly::config::Config;
use fly::db::Db;
use fly::migration::Migration;
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

    let applied_migrations = db.get_applied_migrations()?;

    debug!("migrations in schema table:");
    debug!(
        "{}",
        if applied_migrations.is_empty() {
            "(empty)".to_string()
        } else {
            applied_migrations.join("\n")
        }
    );

    let mut migrations = get_migrations(&config)?;
    migrations.sort_by(|a, b| a.identifier.cmp(&b.identifier));

    debug!("migrations in migrations dir:");
    debug!(
        "{}",
        if migrations.is_empty() {
            "(empty)".to_string()
        } else {
            migrations
                .iter()
                .map(|m| m.identifier.clone())
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    match command {
        Command::Up => {
            let mut any_migrations_run = false;
            for migration in &migrations {
                if !applied_migrations.contains(&migration.identifier.as_str().to_owned()) {
                    info!("applying {}", migration.identifier);
                    debug!("{}", migration.up_sql);
                    db.apply_migration(migration)?;
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
                if migration.identifier == *candidate {
                    info!("reverting {}", migration.identifier);
                    debug!("{}", migration.down_sql);
                    db.rollback_migration(migration)?;
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
                        info!("{} [applied]", migration);
                    } else {
                        info!("{} [pending]", migration);
                    }
                } else {
                    info!("{} ** NO FILE **", migration);
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

fn get_migrations(config: &Config) -> Result<Vec<Migration>> {
    let mut migrations = Vec::new();

    let paths = std::fs::read_dir(&config.migrate_dir).with_context(|| {
        format!(
            "problem reading migration directory ({})",
            &config.migrate_dir.display()
        )
    })?;

    for path in paths {
        let path = path?.path();
        let migration = Migration::from_file(&path)?;
        migrations.push(migration);
    }

    Ok(migrations)
}
