// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod member_controller;
pub mod member_service;
pub mod models;
pub mod user_repository;

pub use member_service::{
    EnrichedMember, LiveUserApplicationService, MemberService, UserApplicationService,
    UserApplicationServiceImpl, VCardData,
};
pub use models::*;
pub use user_repository::{LiveUserRepository, UserRepository, UserRepositoryImpl, UsersStats};

pub mod errors {
    use crate::{errors::Forbidden, util::either::Either};

    #[derive(Debug, thiserror::Error)]
    #[error("Member not found: {0}.")]
    #[repr(transparent)]
    pub struct MemberNotFound(pub String);

    #[derive(Debug, thiserror::Error)]
    pub enum UserDeleteError {
        #[error("Cannot self-remove.")]
        CannotSelfRemove,
        #[error("{0}")]
        Forbidden(#[from] Forbidden),
        #[error("{0:#}")]
        Internal(#[from] anyhow::Error),
    }

    // MARK: Boilerplate

    impl From<Either<Forbidden, anyhow::Error>> for UserDeleteError {
        fn from(either: Either<Forbidden, anyhow::Error>) -> Self {
            match either {
                Either::E1(err) => Self::from(err),
                Either::E2(err) => Self::from(err),
            }
        }
    }
}
