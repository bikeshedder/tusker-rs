use std::ffi::OsStr;
use std::fs::read;
use std::path::Path;

use crate::error::Error;

use super::{collect_unique, Migration, MigrationSource};

/// A [`MigrationSource`] that loads migrations from the filesystem using a glob
/// pattern such as `db/migrations/**/*.sql`.
///
/// Only files with a `.sql` extension are considered. Each file name is parsed
/// as `<number>_<name>.sql`, and the SQL content is read eagerly.
#[derive(Clone, Debug)]
pub struct GlobSource {
    pattern: String,
}

impl GlobSource {
    /// Create a source from a glob pattern.
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }
}

impl MigrationSource for GlobSource {
    fn load(&self) -> Result<Vec<Migration>, Error> {
        let sql_ext = OsStr::new("sql");
        let paths = ::glob::glob(&self.pattern).map_err(|e| {
            Error::Misc(format!("Invalid migrations glob {:?}: {}", self.pattern, e))
        })?;
        collect_unique(paths.filter_map(|entry| match entry {
            Ok(path) => {
                // Skip anything that is not a .sql file (e.g. directories
                // matched by the pattern).
                if path.extension() != Some(sql_ext) {
                    return None;
                }
                Some(load_migration_file(&path))
            }
            Err(e) => Some(Err(Error::Io(
                format!("Error reading migration file {:?}", e.path().display()),
                e.into_error(),
            ))),
        }))
    }
}

fn load_migration_file(path: &Path) -> Result<Migration, Error> {
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy())
        .ok_or_else(|| Error::Misc(format!("Invalid filename: {}", path.display())))?;
    let bytes = read(path)
        .map_err(|e| Error::Io(format!("Error reading SQL file {:?}", path.display()), e))?;
    let sql = String::from_utf8(bytes).map_err(|_| {
        Error::Misc(format!(
            "Migration file is not valid UTF-8: {}",
            path.display()
        ))
    })?;
    Migration::from_stem(&stem, sql).map_err(|e| {
        Error::Misc(format!(
            "Invalid migration file {:?}: {}",
            path.display(),
            e
        ))
    })
}
