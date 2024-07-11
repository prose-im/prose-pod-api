// prose-pod-api
//
// Copyright: 2023, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use sea_orm::DbErr;
use std::fmt;

#[derive(Debug)]
pub enum MutationError {
    DbErr(DbErr),
    EntityNotFound { entity_name: &'static str },
}

impl fmt::Display for MutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DbErr(err) => write!(f, "Database error: {err}"),
            Self::EntityNotFound { entity_name } => write!(f, "Entity not found: {entity_name}"),
        }
    }
}

impl std::error::Error for MutationError {}

impl From<DbErr> for MutationError {
    fn from(value: DbErr) -> Self {
        Self::DbErr(value)
    }
}
