use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "fly", about = "fly: simple database migrations.")]
pub enum Command {
    /// Applies all pending migrations.
    Up,

    /// Rolls back the last migration.
    Down,

    /// Prints the current status of the database.
    Status,

    /// Creates a new migration file.
    New(NewArgs),
}

#[derive(clap::Args, Debug)]
pub struct NewArgs {
    /// The name to use for the migration file, e.g., "create-users"
    pub name: String,
}
