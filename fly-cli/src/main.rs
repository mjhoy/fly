use anyhow::{Context, Result};
use clap::Parser;
use command::Command;
use fly::db::Db;
use fly::planner::ApplicationState;
use fly::{config::Config, planner::get_all_migration_state};
use std::process::exit;
use std::{io::Write, time::SystemTime};
use tracing::{debug, error, info, Level};

mod command;

static MIGRATION_TEMPLATE: &str = "-- up\n\n-- down\n";
static EXAMPLE_ENV: &str = "# fly config
MIGRATE_DIR=migrations
PG_HOST=127.0.0.1
PG_USER=user
PG_PORT=5432
# Remove if no password is set
PG_PASSWORD=1234
PG_DB=db
";

fn startup() -> Result<(Db, Vec<ApplicationState>)> {
    let config = Config::from_env()?;
    let mut db = Db::connect(&config).context("couldn't connect to database")?;
    db.create_migrations_table()
        .context("failed creating migrations table")?;
    let application_state = get_all_migration_state(&mut db, &config.migrate_dir)?;

    Ok((db, application_state))
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let debug = std::env::var("DEBUG").unwrap_or("false".to_string()) == "true";
    let command = Command::parse();
    let level = if debug { Level::DEBUG } else { Level::INFO };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .without_time()
        .with_target(false)
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("setting tracing subscriber failed")?;

    match command {
        Command::Up => {
            let (mut db, application_state) = startup()?;
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
        Command::Down {
            recover,
            ignore_changed,
            name,
        } => {
            let (mut db, application_state) = startup()?;
            if recover && ignore_changed {
                error!("cannot specify both --recover and --ignore-changed, aborting");
                exit(1);
            }
            let migration = if let Some(name) = name {
                match application_state
                    .iter()
                    .find(|application| application.name() == name)
                {
                    Some(s) => Some(s),
                    None => {
                        error!("couldn't find migration {}", name);
                        exit(1);
                    }
                }
            } else {
                application_state
                    .iter()
                    .rfind(|application| !application.is_pending())
            };
            match migration {
                Some(application) => match application {
                    ApplicationState::Applied {
                        definition,
                        application: _,
                    } => {
                        debug!(definition.down_sql);
                        info!("reverting {}", definition.name);
                        db.rollback_migration(definition)?;
                    }
                    ApplicationState::Changed {
                        definition,
                        application,
                    } => {
                        let rollback = if recover {
                            &application.migration
                        } else if ignore_changed {
                            definition
                        } else {
                            error!("{} has changed, aborting. Use the --recover flag to run the down sql stored in the database.", application.migration.name);
                            exit(1)
                        };
                        debug!("{}", rollback.down_sql);
                        info!("reverting {}", rollback.name);
                        db.rollback_migration(rollback)?;
                    }
                    ApplicationState::Removed { application } => {
                        if recover {
                            debug!("{}", application.migration.down_sql);
                            info!("reverting application {}", application.migration.name);
                            db.rollback_migration(&application.migration)?;
                        } else {
                            error!("{} was removed, aborting. Use the --recover flag to run the down sql stored in the database.", application.migration.name);
                            exit(1);
                        }
                    }
                    ApplicationState::Pending { definition } => {
                        error!("can't roll back a pending migration {}", definition.name);
                        exit(1);
                    }
                },
                None => info!("no migrations to revert"),
            }
        }
        Command::Status => {
            let (_, application_state) = startup()?;
            for application in &application_state {
                info!("{}", application);
                debug!("{:?}", application);
            }
        }
        Command::New { name } => {
            let config = Config::from_env()?;
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("time went backwards")
                .as_secs();
            let filename = format!("{}-{}.sql", timestamp, name);
            let path = config.migrate_dir.join(filename);
            let mut file = std::fs::File::create(&path)?;
            file.write_all(MIGRATION_TEMPLATE.as_bytes())?;
            info!("Created file {}", path.display());
        }
        Command::ExampleEnv => println!("{}", EXAMPLE_ENV.trim()),
    }

    Ok(())
}
