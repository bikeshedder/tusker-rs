use std::ffi::OsStr;
use std::fmt;
use std::marker::PhantomData;
use std::path::Path;

use ::rust_embed::RustEmbed;

use crate::error::Error;

use super::{collect_unique, Migration, MigrationSource};

/// A [`MigrationSource`] backed by files embedded into the binary via
/// [`rust-embed`](https://docs.rs/rust-embed).
///
/// ```ignore
/// use rust_embed::RustEmbed;
/// use tusker_migration::RustEmbedSource;
///
/// #[derive(RustEmbed)]
/// #[folder = "db/migrations"]
/// struct Migrations;
///
/// let source = RustEmbedSource::<Migrations>::new();
/// ```
///
/// Only embedded files with a `.sql` extension are considered. Each file name
/// is parsed as `<number>_<name>.sql`.
#[cfg_attr(docsrs, doc(cfg(feature = "rust-embed")))]
pub struct RustEmbedSource<E: RustEmbed> {
    _marker: PhantomData<E>,
}

impl<E: RustEmbed> RustEmbedSource<E> {
    /// Create a source for the given embedded folder type.
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<E: RustEmbed> Default for RustEmbedSource<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: RustEmbed> fmt::Debug for RustEmbedSource<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RustEmbedSource").finish()
    }
}

impl<E: RustEmbed> MigrationSource for RustEmbedSource<E> {
    fn load(&self) -> Result<Vec<Migration>, Error> {
        let sql_ext = OsStr::new("sql");
        collect_unique(E::iter().filter_map(|path| {
            let file_path = Path::new(path.as_ref());
            // Skip anything that is not a .sql file.
            if file_path.extension() != Some(sql_ext) {
                return None;
            }
            Some(load_embedded::<E>(&path))
        }))
    }
}

fn load_embedded<E: RustEmbed>(path: &str) -> Result<Migration, Error> {
    let file_path = Path::new(path);
    let stem = file_path
        .file_stem()
        .map(|stem| stem.to_string_lossy())
        .ok_or_else(|| Error::Misc(format!("Invalid embedded filename: {}", path)))?;
    let file = E::get(path)
        .ok_or_else(|| Error::Misc(format!("Embedded migration file missing: {}", path)))?;
    let sql = String::from_utf8(file.data.into_owned()).map_err(|_| {
        Error::Misc(format!(
            "Embedded migration file is not valid UTF-8: {}",
            path
        ))
    })?;
    Migration::from_stem(&stem, sql)
        .map_err(|e| Error::Misc(format!("Invalid embedded migration file {:?}: {}", path, e)))
}
