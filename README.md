fly
===

A simple CLI database migration tool for postgresql.

## Configuration

Fly expects the following env variables set. It will also use `dotenv`
to look in a `.env` file.

- `MIGRATE_DIR`: Path to your migrations (e.g., `db/migrate`).
- `PG_USER`
- `PG_PASSWORD` (optional)
- `PG_HOST`
- `PG_PORT`
- `PG_DB`

## Subcommands

- `up`: Applies all pending migrations.
- `down`: Rolls back the last migration.
- `status`: Prints the current status of the database.
- `new`: Creates a new migration file.
