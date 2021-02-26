use std::fs::{DirEntry, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use diesel::backend::Backend;
use diesel::connection::BoxableConnection;
use diesel::migration::{Migration, MigrationMetadata, MigrationSource, MigrationVersion};
use migrations_internals::TomlMetadata;

use crate::errors::{MigrationError, RunMigrationsError};

#[derive(Clone)]
pub struct FileBasedMigrations {
    base_path: PathBuf,
}

impl FileBasedMigrations {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, MigrationError> {
        for dir in migrations_directories(path.as_ref()) {
            let path = dir?.path();
            if !migrations_internals::valid_sql_migration_directory(&path) {
                return Err(MigrationError::UnknownMigrationFormat(path));
            }
        }
        Ok(Self {
            base_path: path.as_ref().to_path_buf(),
        })
    }

    pub fn find_migrations_directory() -> Result<Self, MigrationError> {
        Self::find_migrations_directory_in_path(&std::env::current_dir()?)
    }

    pub fn find_migrations_directory_in_path(
        path: impl AsRef<Path>,
    ) -> Result<Self, MigrationError> {
        let migrations_directory = search_for_migrations_directory(path.as_ref())?;
        Self::from_path(&migrations_directory)
    }

    #[doc(hidden)]
    pub fn path(&self) -> &Path {
        &self.base_path
    }
}

fn search_for_migrations_directory(path: &Path) -> Result<PathBuf, MigrationError> {
    migrations_internals::search_for_migrations_directory(path)
        .ok_or_else(|| MigrationError::MigrationDirectoryNotFound(path.to_path_buf()))
}

fn migrations_directories<'a>(
    path: &'a Path,
) -> impl Iterator<Item = Result<DirEntry, MigrationError>> + 'a {
    migrations_internals::migrations_directories(path).map(move |e| e.map_err(Into::into))
}

fn migrations_in_directory<'a, DB: Backend>(
    path: &'a Path,
) -> impl Iterator<Item = Result<SqlFileMigration, MigrationError>> + 'a {
    migrations_directories(path).map(|entry| SqlFileMigration::from_path(&entry?.path()))
}

impl<DB: Backend> MigrationSource<DB> for FileBasedMigrations {
    fn migrations(
        &self,
    ) -> Result<Vec<Box<dyn Migration<DB>>>, Box<dyn std::error::Error + Send + Sync + 'static>>
    {
        migrations_in_directory::<DB>(&self.base_path)
            .map(|r| Ok(Box::new(r?) as Box<dyn Migration<DB>>))
            .collect()
    }
}

struct SqlFileMigration {
    base_path: PathBuf,
    metadata: TomlMetadataWrapper,
    version: String,
}

impl SqlFileMigration {
    fn from_path(path: &Path) -> Result<Self, MigrationError> {
        if migrations_internals::valid_sql_migration_directory(path) {
            let version = version_from_path(path)?;
            let metadata = TomlMetadataWrapper(
                TomlMetadata::read_from_file(&path.join("metadata.toml")).unwrap_or_default(),
            );
            Ok(Self {
                base_path: path.to_path_buf(),
                metadata,
                version,
            })
        } else {
            Err(MigrationError::UnknownMigrationFormat(path.to_path_buf()))
        }
    }
}

impl<DB: Backend> Migration<DB> for SqlFileMigration {
    fn version<'a>(&'a self) -> MigrationVersion<'a> {
        MigrationVersion::from(&self.version)
    }

    fn run(
        &self,
        conn: &dyn BoxableConnection<DB>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(run_sql_from_file(
            Migration::<DB>::version(self),
            conn,
            &self.base_path.join("up.sql"),
        )?)
    }

    fn revert(
        &self,
        conn: &dyn BoxableConnection<DB>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(run_sql_from_file(
            Migration::<DB>::version(self),
            conn,
            &self.base_path.join("up.sql"),
        )?)
    }

    fn metadata(&self) -> &dyn MigrationMetadata {
        &self.metadata
    }
}

#[derive(Default)]
#[doc(hidden)]
pub struct TomlMetadataWrapper(TomlMetadata);

impl TomlMetadataWrapper {
    pub const fn new(run_in_transaction: bool) -> Self {
        Self(TomlMetadata::new(run_in_transaction))
    }
}

impl MigrationMetadata for TomlMetadataWrapper {
    fn run_in_transaction(&self) -> bool {
        self.0.run_in_transaction
    }
}

fn run_sql_from_file<DB: Backend>(
    version: MigrationVersion<'_>,
    conn: &dyn BoxableConnection<DB>,
    path: &Path,
) -> Result<(), RunMigrationsError> {
    let map_io_err =
        |e| RunMigrationsError::MigrationError(version.into_owned(), MigrationError::from(e));

    let mut sql = String::new();
    let mut file = File::open(path).map_err(map_io_err)?;
    file.read_to_string(&mut sql).map_err(map_io_err)?;

    if sql.is_empty() {
        return Err(RunMigrationsError::EmptyMigration(version.into_owned()));
    }

    conn.batch_execute(&sql)
        .map_err(|e| RunMigrationsError::QueryError(version.into_owned(), e))?;
    Ok(())
}

fn version_from_path(path: &Path) -> Result<String, MigrationError> {
    Ok(migrations_internals::version_from_path(path)
        .ok_or_else(|| MigrationError::UnknownMigrationFormat(path.to_path_buf()))??)
}

// #[cfg(test)]
// mod tests {
//     extern crate tempfile;

//     use super::*;

//     use self::tempfile::Builder;
//     use std::fs;

//     #[test]
//     fn migration_directory_not_found_if_no_migration_dir_exists() {
//         let dir = Builder::new().prefix("diesel").tempdir().unwrap();

//         assert_eq!(
//             Err(MigrationError::MigrationDirectoryNotFound(
//                 dir.path().into()
//             )),
//             search_for_migrations_directory(dir.path())
//         );
//     }

//     #[test]
//     fn migration_directory_defaults_to_pwd_slash_migrations() {
//         let dir = Builder::new().prefix("diesel").tempdir().unwrap();
//         let temp_path = dir.path().canonicalize().unwrap();
//         let migrations_path = temp_path.join("migrations");

//         fs::create_dir(&migrations_path).unwrap();

//         assert_eq!(
//             Ok(migrations_path),
//             search_for_migrations_directory(&temp_path)
//         );
//     }

//     #[test]
//     fn migration_directory_checks_parents() {
//         let dir = Builder::new().prefix("diesel").tempdir().unwrap();
//         let temp_path = dir.path().canonicalize().unwrap();
//         let migrations_path = temp_path.join("migrations");
//         let child_path = temp_path.join("child");

//         fs::create_dir(&child_path).unwrap();
//         fs::create_dir(&migrations_path).unwrap();

//         assert_eq!(
//             Ok(migrations_path),
//             search_for_migrations_directory(&child_path)
//         );
//     }

//     #[test]
//     fn migration_paths_in_directory_ignores_files() {
//         let dir = Builder::new().prefix("diesel").tempdir().unwrap();
//         let temp_path = dir.path().canonicalize().unwrap();
//         let migrations_path = temp_path.join("migrations");
//         let file_path = migrations_path.join("README.md");

//         fs::create_dir(&migrations_path).unwrap();
//         fs::File::create(&file_path).unwrap();

//         assert_eq!(
//             0,
//             migration_paths_in_directory(&migrations_path)
//                 .unwrap()
//                 .len()
//         );
//     }

//     #[test]
//     fn migration_paths_in_directory_ignores_dot_directories() {
//         let dir = Builder::new().prefix("diesel").tempdir().unwrap();
//         let temp_path = dir.path().canonicalize().unwrap();
//         let migrations_path = temp_path.join("migrations");
//         let dot_path = migrations_path.join(".hidden_dir");

//         fs::create_dir(&migrations_path).unwrap();
//         fs::create_dir(&dot_path).unwrap();

//         assert_eq!(
//             0,
//             migration_paths_in_directory(&migrations_path)
//                 .unwrap()
//                 .len()
//         );
//     }
// }
