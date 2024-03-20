# Changelog

## Unreleased

### Added

- (Breaking) The `up`/`down` sql strings are now tracked in the
  database. This allows fly to know when a migration has been changed,
  and how to un-apply an old or removed migration.
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
