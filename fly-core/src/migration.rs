use std::{cmp::Ordering, time::SystemTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub up_sql: String,
    pub down_sql: String,
    pub name: String,
}

impl PartialOrd for Migration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for Migration {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MigrationMeta {
    pub id: i32,
    pub created_at: SystemTime,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MigrationWithMeta {
    pub migration: Migration,
    pub meta: MigrationMeta,
}
