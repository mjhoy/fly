fly
===

A simple CLI database migration tool for postgresql. Very much
work-in-progress at the moment.

## Installing

Available on crates.io as [fly-migrate][fly-migrate]:

```
$ cargo install fly-migrate
# installs `fly` to `~/.cargo/bin`:
$ fly --help
```

## Configuration

Fly expects the following env variables set. It will also use `dotenv`
to look in a `.env` file.

- `MIGRATE_DIR`: Path to your migrations (e.g., `db/migrate`).
- `PG_USER`
- `PG_PASSWORD` (optional)
- `PG_HOST`
- `PG_PORT`
- `PG_DB`

You can also directly set a `PG_CONNECTION_STRING` instead of the
individual `PG_` variables.

## Subcommands

- `up`: Applies all pending migrations.
- `down`: Rolls back the last migration.
- `status`: Prints the current status of the database.
- `new`: Creates a new migration file.

[fly-migrate]: https://crates.io/crates/fly-migrate
