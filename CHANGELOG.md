# Changelog

## [0.2.1] 2024-03-23

### Fixed

- Fixed the readme config in the new workspace setup.

## [0.2.0] 2024-03-23

### Added

- (Breaking) The `up`/`down` sql strings are now tracked in the
  database. This allows fly to know when a migration has been changed,
  and how to un-apply an old or removed migration.
- `fly down` has the ability to recover from changed or removed
  migrations with the `--recover` flag, along with other improvements.
- `fly example-env` outputs an example `.env` file.
- Integration tests and github CI.

### Changed

- Broke out a separate `fly-migrate-core` library crate.
- Touched up logging and errors a bit.

## [0.1.2] 2024-03-11

### Added

- Now accepts a `PG_CONNECTION_STRING` environment variable.
- Split into a basic library + binary crate structure.
- Added some sanity integration tests around the executable.

## [0.1.1] 2023-03-30

Improved error messages.

## [0.1.0] 2023-03-29

Initial release.
