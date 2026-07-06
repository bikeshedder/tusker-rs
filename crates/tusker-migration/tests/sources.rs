//! Tests for the built-in [`MigrationSource`] implementations.
//!
//! Tests run with the crate root as the working directory, so the relative
//! fixture paths below resolve against `crates/tusker-migration`.

use tusker_migration::{GlobSource, MigrationSource};

/// `(number, name, sql)` triples make comparing sources easy — and since the
/// SHA-512 hash is derived purely from the SQL, equal triples imply equal
/// hashes.
fn triples(source: &dyn MigrationSource) -> Vec<(i32, String, String)> {
    let mut migrations = source.load().expect("source should load");
    migrations.sort_by_key(|migration| migration.number);
    migrations
        .into_iter()
        .map(|migration| (migration.number, migration.name, migration.sql))
        .collect()
}

#[test]
fn glob_source_loads_sql_files_and_skips_others() {
    let source = GlobSource::new("tests/fixtures/migrations/*.sql");
    let migrations = triples(&source);
    assert_eq!(
        migrations,
        vec![
            (1, "first".to_string(), "CREATE TABLE first (id int);\n".to_string()),
            (
                2,
                "second".to_string(),
                "ALTER TABLE first ADD COLUMN name text;\n".to_string()
            ),
        ]
    );
}

#[test]
fn glob_source_rejects_duplicate_numbers() {
    let source = GlobSource::new("tests/fixtures/duplicates/*.sql");
    let err = source.load().expect_err("duplicate numbers must be rejected");
    assert!(
        err.to_string().contains("multiple files for number 1"),
        "unexpected error: {err}"
    );
}

#[cfg(feature = "rust-embed")]
mod rust_embed {
    use ::rust_embed::RustEmbed;
    use tusker_migration::{MigrationSource, RustEmbedSource};

    use super::triples;

    #[derive(RustEmbed)]
    #[folder = "tests/fixtures/migrations"]
    struct Migrations;

    #[test]
    fn rust_embed_source_matches_glob_source() {
        let embed = RustEmbedSource::<Migrations>::new();
        let glob = tusker_migration::GlobSource::new("tests/fixtures/migrations/*.sql");
        // Both sources must yield the same migrations (and therefore the same
        // hashes), whether loaded from disk or embedded into the binary.
        assert_eq!(triples(&embed), triples(&glob));
    }

    #[test]
    fn rust_embed_source_skips_non_sql_files() {
        let embed = RustEmbedSource::<Migrations>::new();
        let migrations = embed.load().expect("embedded source should load");
        assert_eq!(migrations.len(), 2);
    }
}
