use crate::db::Db;
use crate::error::Result;
use crate::file;
use crate::migration::{Migration, MigrationWithMeta};
use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationState {
    Pending {
        definition: Migration,
    },
    Applied {
        definition: Migration,
        application: MigrationWithMeta,
    },
    Changed {
        definition: Migration,
        application: MigrationWithMeta,
    },
    Removed {
        application: MigrationWithMeta,
    },
}

impl ApplicationState {
    pub fn is_pending(&self) -> bool {
        matches!(self, ApplicationState::Pending { .. })
    }

    pub fn is_applied(&self) -> bool {
        matches!(self, ApplicationState::Applied { .. })
    }

    pub fn is_changed(&self) -> bool {
        matches!(self, ApplicationState::Changed { .. })
    }

    pub fn is_removed(&self) -> bool {
        matches!(self, ApplicationState::Removed { .. })
    }

    pub fn name(&self) -> &str {
        match self {
            ApplicationState::Pending { definition } => &definition.name,
            ApplicationState::Applied {
                definition,
                application: _,
            } => &definition.name,
            ApplicationState::Changed {
                definition,
                application: _,
            } => &definition.name,
            ApplicationState::Removed { application } => &application.migration.name,
        }
    }
}

impl Display for ApplicationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationState::Pending { definition: file } => write!(f, "{} [pending]", file.name),
            ApplicationState::Applied {
                definition: file,
                application: _,
            } => {
                write!(f, "{} [applied]", file.name)
            }
            ApplicationState::Changed {
                definition: file,
                application: _,
            } => write!(f, "{} ** CHANGED **", file.name),
            ApplicationState::Removed { application: db } => {
                write!(f, "{} ** NO FILE **", db.migration.name)
            }
        }
    }
}

pub fn get_all_migration_state(
    db: &mut Db,
    migrate_dir: impl AsRef<Path>,
) -> Result<Vec<ApplicationState>> {
    let definitions = file::list(migrate_dir.as_ref())?;
    let applications = db.list()?;
    Ok(get_all_migration_state_impl(definitions, applications))
}

fn get_all_migration_state_impl(
    definitions: Vec<Migration>,
    applications: Vec<MigrationWithMeta>,
) -> Vec<ApplicationState> {
    let definitions = definitions
        .into_iter()
        .map(|m| (m.name.clone(), m))
        .collect::<HashMap<String, Migration>>();
    let applications = applications
        .into_iter()
        .map(|m| (m.migration.name.clone(), m))
        .collect::<HashMap<String, MigrationWithMeta>>();

    let mut all_names = definitions
        .values()
        .map(|v| v.name.clone())
        .chain(applications.values().map(|v| v.migration.name.clone()))
        .collect::<Vec<String>>();
    all_names.sort();
    all_names.dedup();

    all_names
        .iter()
        .map(|name| {
            let definition = definitions.get(name);
            let application = applications.get(name);

            match (definition, application) {
                (None, Some(application)) => ApplicationState::Removed {
                    application: application.clone(),
                },
                (Some(definition), None) => ApplicationState::Pending {
                    definition: definition.clone(),
                },
                (Some(definition), Some(application)) => {
                    if application.migration == *definition {
                        ApplicationState::Applied {
                            definition: definition.clone(),
                            application: application.clone(),
                        }
                    } else {
                        ApplicationState::Changed {
                            definition: definition.clone(),
                            application: application.clone(),
                        }
                    }
                }
                (None, None) => unreachable!(),
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
    use crate::migration::MigrationMeta;
    use rand::seq::SliceRandom;
    use std::time::SystemTime;

    use super::*;

    #[test]
    fn test_get_all_migration_state_empty() {
        let result = get_all_migration_state_impl(Vec::new(), Vec::new());
        assert_eq!(result, Vec::new());
    }

    #[test]
    fn test_get_all_migration_state_pending() {
        let definition = build_migration("migration", "up", "down");
        let result = get_all_migration_state_impl(vec![definition.clone()], Vec::new());
        assert_eq!(
            result,
            vec![ApplicationState::Pending {
                definition: definition
            }]
        );
    }

    #[test]
    fn test_get_all_migration_state_removed() {
        let application = build_migration_meta("migration", "up", "down");
        let result = get_all_migration_state_impl(Vec::new(), vec![application.clone()]);
        assert_eq!(
            result,
            vec![ApplicationState::Removed {
                application: application
            }]
        );
    }

    #[test]
    fn test_get_all_migration_state_applied() {
        let definition = build_migration("migration", "up", "down");
        let application = build_migration_meta("migration", "up", "down");
        let result =
            get_all_migration_state_impl(vec![definition.clone()], vec![application.clone()]);
        assert_eq!(
            result,
            vec![ApplicationState::Applied {
                definition: definition,
                application: application
            }]
        );
    }

    #[test]
    fn test_get_all_migration_up_state_changed() {
        let definition = build_migration("migration", "up-changed", "down");
        let application = build_migration_meta("migration", "up", "down");
        let result =
            get_all_migration_state_impl(vec![definition.clone()], vec![application.clone()]);
        assert_eq!(
            result,
            vec![ApplicationState::Changed {
                definition: definition,
                application: application
            }]
        );
    }

    #[test]
    fn test_get_all_migration_down_state_changed() {
        let definition = build_migration("migration", "up", "down-changed");
        let application = build_migration_meta("migration", "up", "down");
        let result =
            get_all_migration_state_impl(vec![definition.clone()], vec![application.clone()]);
        assert_eq!(
            result,
            vec![ApplicationState::Changed {
                definition: definition,
                application: application
            }]
        );
    }

    #[test]
    fn test_get_all_migration_state_multiple() {
        let definition_a = build_migration("1-migration", "1-up", "1-down");
        let definition_b = build_migration("2-migration", "2-up", "2-down");
        let definition_c = build_migration("3-migration", "3-up", "3-down");

        let application_a = build_migration_meta("1-migration", "1-up", "1-down");
        let application_b = build_migration_meta("2-migration", "2-up-changed", "2-down");
        let application_c = build_migration_meta("4-migration", "4-up", "4-down");

        let mut definitions = vec![
            definition_a.clone(),
            definition_b.clone(),
            definition_c.clone(),
        ];
        let mut applications = vec![
            application_a.clone(),
            application_b.clone(),
            application_c.clone(),
        ];

        // The particular order of definitions/applications should not
        // matter, as they are keyed and sorted by name.
        let mut rng = rand::thread_rng();
        definitions.shuffle(&mut rng);
        applications.shuffle(&mut rng);

        let result = get_all_migration_state_impl(definitions, applications);
        assert_eq!(
            result,
            vec![
                ApplicationState::Applied {
                    definition: definition_a,
                    application: application_a
                },
                ApplicationState::Changed {
                    definition: definition_b,
                    application: application_b
                },
                ApplicationState::Pending {
                    definition: definition_c
                },
                ApplicationState::Removed {
                    application: application_c
                },
            ]
        );
    }

    fn build_migration(name: &'static str, up: &'static str, down: &'static str) -> Migration {
        Migration {
            up_sql: up.to_string(),
            down_sql: down.to_string(),
            name: name.to_string(),
        }
    }

    fn build_migration_meta(
        name: &'static str,
        up: &'static str,
        down: &'static str,
    ) -> MigrationWithMeta {
        let now = SystemTime::now();

        MigrationWithMeta {
            meta: MigrationMeta {
                id: 123,
                created_at: now,
            },
            migration: build_migration(name, up, down),
        }
    }
}
