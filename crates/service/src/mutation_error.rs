// prose-pod-api
//
// Copyright: 2023–2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Debug, thiserror::Error)]
pub enum MutationError {
    #[error("Entity not found: {entity_name}")]
    EntityNotFound { entity_name: &'static str },
    #[error("Database error: {0}")]
    DbErr(#[from] sea_orm::DbErr),
}
