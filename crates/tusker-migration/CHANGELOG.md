# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `MigrationSource` trait decoupling migration loading from the runner, with a
  filesystem `GlobSource` and an embedded `RustEmbedSource` (behind the new opt-in
  `rust-embed` feature).

### Changed

- **Breaking:** `cli::cmd` and `cli::run` now take a `&dyn MigrationSource`
  argument; the source is chosen in code rather than via CLI flags.
- **Breaking:** removed the `--migrations-dir` command-line option. Embedding
  applications supply a `MigrationSource`; the `tusker` CLI now loads migrations
  from the `migrations.filename` glob in its config.
- **Breaking:** renamed the public migration types: The source-loaded item is
  now `Migration` (was `MigrationFile`), the database row is `AppliedMigration`
  (was `DbMigration`), and the reconciled pair is `MigrationState`.

## [0.1.1] - 2026-05-23

### Changed

- Improve crate metadata with repository, keywords, and categories

## [0.1.0] - 2026-05-22

### Added

- Initial release

[unreleased]: https://github.com/bikeshedder/tusker/compare/tusker-migration-v0.1.1...HEAD
[0.1.1]: https://github.com/bikeshedder/tusker/releases/tag/tusker-migration-v0.1.1
[0.1.0]: https://github.com/bikeshedder/tusker/releases/tag/tusker-migration-v0.1.0
