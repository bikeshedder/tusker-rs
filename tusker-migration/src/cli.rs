use std::io::Write;
use std::path::Path;

use clap::{Args, Subcommand};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::db::Database;
use crate::error::Error;
use crate::file::load_migration_files;
use crate::models::{combine_migrations, MigrationStatus};

#[derive(Debug, Args)]
#[clap(about = "Manage database migrations")]
pub struct Command {
    #[clap(
        long,
        short,
        value_name = "DIRECTORY",
        help = "Directory containing the migrations",
        default_value = "db/migrations"
    )]
    migrations_dir: String,

    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[derive(Debug, Subcommand)]
pub enum Subcommands {
    #[clap(aliases = &["ls", "list"], about = "List migrations and show their current status")]
    Status,

    #[clap(aliases = &["history"], about = "Show migration log")]
    Log,

    #[clap(aliases = &["apply"], about = "Run migrations on the database")]
    Run(RunArgs),

    Check,

    #[clap(about = "Fix database migration")]
    Fix(FixArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[clap(
        long,
        short,
        help = "Number of the migration to be run. If no number is provided all outstanding migrations are run."
    )]
    number: Option<i32>,
}

#[derive(Debug, Args)]
pub struct FixArgs {
    #[clap(value_name = "NUMBER")]
    number: i32,
}

pub async fn run(pg_config: &tokio_postgres::Config, cmd: &Command) -> Result<(), Error> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_bold = ColorSpec::new();
    color_bold.set_bold(true);
    let mut color_ok = ColorSpec::new();
    color_ok.set_fg(Some(Color::Green));
    color_ok.set_bold(true);
    let mut color_error = ColorSpec::new();
    color_error.set_fg(Some(Color::Red));
    color_error.set_bold(true);
    let mut color_new = ColorSpec::new();
    color_new.set_fg(Some(Color::Cyan));
    color_new.set_bold(true);
    let mut color_modified = ColorSpec::new();
    color_modified.set_fg(Some(Color::Yellow));
    color_modified.set_bold(true);

    let migration_files = load_migration_files(Path::new(&cmd.migrations_dir))?;

    let mut db = Database::connect(pg_config)
        .await
        .map_err(|e| Error::Pg("PostgreSQL connection failed".into(), e))?;
    let migration_table_exists = db
        .migration_table_exists()
        .await
        .map_err(|e| Error::Pg("Checking status table failed".into(), e))?;

    let db_migrations = match migration_table_exists {
        true => db
            .get_migrations()
            .await
            .map_err(|e| Error::Pg("Unable to load already applied migrations".into(), e))?,
        false => Vec::new(),
    };

    let migrations = combine_migrations(&migration_files, &db_migrations);

    match &cmd.subcommand {
        Subcommands::Status => {
            if !migration_table_exists {
                writeln!(
                    &mut stdout,
                    "Migration table missing. Migrations were probably never run."
                )?;
            }
            writeln!(
                &mut stdout,
                "   # Name                                             Status"
            )?;
            writeln!(
                &mut stdout,
                "----------------------------------------------------------------------------"
            )?;
            for migration in migrations.iter() {
                write!(&mut stdout, "{:04} ", migration.number)?;
                match migration.get_status() {
                    MigrationStatus::Ok(migration_file, _) => {
                        stdout.set_color(&color_bold)?;
                        write!(&mut stdout, "{:48} ", migration_file.name)?;
                        stdout.set_color(&color_ok)?;
                        writeln!(&mut stdout, "Ok")?;
                        stdout.reset()?;
                    }
                    MigrationStatus::Mismatch(migration_file, db_migration) => {
                        stdout.set_color(&color_bold)?;
                        write!(&mut stdout, "{:48} ", migration_file.name)?;
                        stdout.set_color(&color_modified)?;
                        write!(&mut stdout, "Mismatch:")?;
                        if migration_file.name != db_migration.name {
                            write!(&mut stdout, " name")?;
                        }
                        if migration_file.hash != db_migration.hash {
                            write!(&mut stdout, " hash")?;
                        }
                        writeln!(&mut stdout)?;
                        stdout.reset()?;
                    }
                    MigrationStatus::NotApplied(migration_file) => {
                        stdout.set_color(&color_bold)?;
                        write!(&mut stdout, "{:48} ", migration_file.name)?;
                        stdout.set_color(&color_new)?;
                        write!(&mut stdout, "New")?;
                        stdout.reset()?;
                        writeln!(&mut stdout)?;
                    }
                    MigrationStatus::FileMissing(db_migration) => {
                        // Migration is part of the database but the migration file
                        // does no longer exist.
                        stdout.set_color(&color_bold)?;
                        write!(&mut stdout, "{:48} ", db_migration.name)?;
                        stdout.set_color(&color_error)?;
                        writeln!(&mut stdout, "Migration file missing")?;
                        stdout.reset()?;
                    }
                }
            }
        }
        Subcommands::Log => {
            let log = db
                .get_migration_log()
                .await
                .map_err(|e| Error::Pg("Error fetching migration log".into(), e))?;
            writeln!(
                &mut stdout,
                "Timestamp                         Operation    # Name                            "
            )?;
            writeln!(
                &mut stdout,
                "---------------------------------------------------------------------------------"
            )?;
            for log_entry in log {
                write!(stdout, "{} ", log_entry.timestamp)?;
                match log_entry.operation.as_str() {
                    "apply" => {
                        stdout.set_color(&color_ok)?;
                    }
                    "update" => {
                        stdout.set_color(&color_modified)?;
                    }
                    "delete" => {
                        stdout.set_color(&color_error)?;
                    }
                    "fake" => {
                        stdout.set_color(&color_modified)?;
                    }
                    _ => {}
                }
                write!(stdout, "{:10}", log_entry.operation)?;
                stdout.reset()?;
                write!(stdout, "{:04} ", log_entry.number)?;
                stdout.set_color(&color_bold)?;
                writeln!(stdout, "{} ", log_entry.name)?;
                stdout.reset()?;
            }
        }
        Subcommands::Check => {
            for migration in migrations {
                if let MigrationStatus::Ok(_, _) = migration.get_status() {
                    continue;
                } else {
                    stdout.set_color(&color_error)?;
                    writeln!(
                        stdout,
                        "Not all migrations cleanly applied. See `status` for more details"
                    )?;
                    stdout.reset()?;
                    std::process::exit(1);
                }
            }
            stdout.set_color(&color_ok)?;
            writeln!(stdout, "All migrations applied")?;
            stdout.reset()?;
            return Ok(());
        }
        Subcommands::Run(_args) => {
            if !migration_table_exists {
                writeln!(stdout, "Creating migration table, views and functions...")?;
                db.init().await.map_err(|e| {
                    Error::Pg(
                        "Unable to create migration table, views and functions".into(),
                        e,
                    )
                })?;
            }
            // FIXME make sure there are no modified or missing migration files first (!)
            // FIXME add support for running only a specific migration
            //let number = matches.value_of("number").map(|s| s.parse::<i32>());
            //println!("NUMBER={:?}", number);
            for migration in migrations.iter() {
                match migration.get_status() {
                    MigrationStatus::Ok(_, _) => {}
                    MigrationStatus::Mismatch(_, _) => {
                        return Err(Error::Misc(
                            "Migration file mismatch found. See `status` for more details".into(),
                        ));
                    }
                    MigrationStatus::NotApplied(migration_file) => {
                        write!(stdout, "Applying migration {}: ", migration_file.number)?;
                        stdout.set_color(&color_bold)?;
                        write!(stdout, "{}", migration_file.name)?;
                        stdout.reset()?;
                        writeln!(stdout)?;
                        let sql = migration_file.read()?;
                        db.apply_migration(migration_file, sql.as_str())
                            .await
                            .map_err(|e| {
                                Error::Pg(
                                    format!(
                                        "Applying migration file {:?} failed",
                                        migration_file.path
                                    ),
                                    e,
                                )
                            })?;
                    }
                    MigrationStatus::FileMissing(_) => {
                        return Err(Error::Misc(
                            "Migration file missing. See `status` for more details".into(),
                        ));
                    }
                }
            }
            stdout.set_color(&color_ok)?;
            writeln!(stdout, "Done.")?;
            stdout.reset()?;
        }
        Subcommands::Fix(args) => {
            let index = migrations.binary_search_by_key(&args.number, |m| m.number);
            if let Ok(index) = index {
                let migration = migrations.get(index).unwrap();
                match migration.get_status() {
                    MigrationStatus::Ok(_, _) => {
                        return Err(Error::Misc(format!(
                            "Migration does not require fixing: {}",
                            args.number
                        )));
                    }
                    MigrationStatus::Mismatch(migration_file, _) => {
                        db.update_migration(migration_file)
                            .await
                            .map_err(|e| Error::Pg("Fixing migration failed".into(), e))?;
                    }
                    MigrationStatus::NotApplied(migration_file) => {
                        // XXX should this be a separate command?
                        db.fake_migration(migration_file)
                            .await
                            .map_err(|e| Error::Pg("Fixing migration failed".into(), e))?;
                    }
                    MigrationStatus::FileMissing(migration_file) => {
                        db.remove_migration(migration_file.number)
                            .await
                            .map_err(|e| Error::Pg("Fixing migration failed".into(), e))?;
                    }
                }
            } else {
                return Err(Error::Misc(format!(
                    "Migration number does not exist: {}",
                    args.number
                )));
            }
        }
    }
    Ok(())
}
