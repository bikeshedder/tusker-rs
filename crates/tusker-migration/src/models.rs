use std::collections::BTreeMap;

use crate::db::AppliedMigration;
use crate::source::Migration;

pub(crate) enum MigrationStatus<'a> {
    Ok(&'a Migration, &'a AppliedMigration),
    Mismatch(&'a Migration, &'a AppliedMigration),
    NotApplied(&'a Migration),
    FileMissing(&'a AppliedMigration),
}

#[derive(Debug)]
pub(crate) struct MigrationState {
    pub(crate) number: i32,
    pub(crate) migration: Option<Migration>,
    pub(crate) applied: Option<AppliedMigration>,
}

impl MigrationState {
    pub(crate) fn get_status(&self) -> MigrationStatus<'_> {
        match (&self.migration, &self.applied) {
            (Some(migration), Some(applied)) => {
                if migration.name == applied.name && migration.hash == applied.hash {
                    MigrationStatus::Ok(migration, applied)
                } else {
                    MigrationStatus::Mismatch(migration, applied)
                }
            }
            (Some(migration), None) => MigrationStatus::NotApplied(migration),
            (None, Some(applied)) => MigrationStatus::FileMissing(applied),
            (None, None) => {
                panic!("Neither 'migration' nor 'applied' set. This should never happen.");
            }
        }
    }
}

pub(crate) fn combine_migrations(
    migrations: &[Migration],
    applied_migrations: &[AppliedMigration],
) -> Vec<MigrationState> {
    let mut map: BTreeMap<i32, MigrationState> = BTreeMap::new();
    for migration in migrations {
        let _ = map.insert(
            migration.number,
            MigrationState {
                number: migration.number,
                migration: Some(migration.clone()),
                applied: None,
            },
        );
    }
    for applied in applied_migrations {
        if let Some(state) = map.get_mut(&applied.number) {
            let _ = state.applied.replace(applied.clone());
        } else {
            let _ = map.insert(
                applied.number,
                MigrationState {
                    number: applied.number,
                    migration: None,
                    applied: Some(applied.clone()),
                },
            );
        }
    }
    map.into_values().collect()
}
