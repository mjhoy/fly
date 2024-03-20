use anyhow::{Context, Result};
use clap::Parser;
use command::Command;
use fly::db::Db;
use fly::planner::ApplicationState;
use fly::{config::Config, planner::get_all_migration_state};
use std::{io::Write, time::SystemTime};
use tracing::{debug, error, info, Level};

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

    let application_state = get_all_migration_state(&mut db, &config.migrate_dir)?;

    debug!("migration state:");
    debug!(
        "{}",
        if application_state.is_empty() {
            "(no migrations defined or applied)".to_string()
        } else {
            application_state
                .iter()
                .map(|application| format!("{}", &application))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    match command {
        Command::Up => {
            let mut any_migrations_run = false;
            for application in &application_state {
                match application {
                    ApplicationState::Pending { definition } => {
                        info!("applying {}", definition.name);
                        debug!("{}", definition.up_sql);
                        db.run(definition)?;
                        any_migrations_run = true;
                    }
                    _ => continue,
                }
            }
            if !any_migrations_run {
                info!("database is up to date");
            }
        }
        Command::Down => {
            let last_non_pending = application_state
                .iter()
                .rfind(|application| !application.is_pending());
            match last_non_pending {
                Some(application) => match application {
                    ApplicationState::Applied {
                        definition,
                        application: _,
                    } => {
                        debug!(definition.down_sql);
                        info!("reverting {}", definition.name);
                        db.rollback_migration(definition)?;
                    }
                    _ => {
                        error!("expected clean migration history, got {}", application)
                    }
                },
                None => info!("no migrations to revert"),
            }
        }
        Command::Status => {
            for application in &application_state {
                println!("{}", application);
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
