use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version,
    name = "fly",
    about = "fly: simple postgresql database migrations."
)]
pub enum Command {
    /// Applies all pending migrations.
    Up,

    /// Rolls back the last migration.
    Down {
        /// If the migration is changed or removed, attempt to roll back using the down sql
        /// string stored in the database. Cannot be used with `--ignore-changed`.
        #[clap(short, long, default_value_t = false)]
        recover: bool,

        /// If the migration is changed, run the down sql defined in the migration file.
        /// Cannot be used with `--recover`.
        #[clap(short, long, default_value_t = false)]
        ignore_changed: bool,

        /// The name of the migration to roll back. If not provided, the default is to select
        /// the latest non-pending migration.
        name: Option<String>,
    },

    /// Prints the current status of the database.
    Status,

    /// Creates a new migration file.
    New {
        /// The name to use for the migration file, e.g., "create-users"
        name: String,
    },
}
