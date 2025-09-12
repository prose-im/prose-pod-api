// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::borrow::Cow;
use std::sync::{Arc, RwLock};

use serdev::Serialize;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::models::SerializableSecretString;

pub const FACTORY_RESET_CONFIRMATION_CODE_LENGTH: usize = 16;

#[derive(Debug, Clone, Default)]
pub struct FactoryResetService {
    confirmation_code: Arc<RwLock<Option<FactoryResetConfirmationCode>>>,
}

impl FactoryResetService {
    pub(super) fn get_confirmation_code(&self) -> FactoryResetConfirmationCode {
        // Generate a random 16-characters-long string.
        let confirmation = FactoryResetConfirmationCode::new();

        // Store the code for later confirmation.
        // WARN: This means two people can’t ask for a factory reset
        //   concurrently… but who cares?
        *self.confirmation_code.write().unwrap() = Some(confirmation.clone());

        confirmation
    }

    pub(super) fn is_confirmation_code_valid(&self, code: &FactoryResetConfirmationCode) -> bool {
        match self.confirmation_code.read().unwrap().as_ref() {
            Some(valid) => valid == code,
            None => false,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, serdev::Deserialize)]
#[serde(validate = "Validate::validate")]
#[repr(transparent)]
pub struct FactoryResetConfirmationCode(SerializableSecretString);

impl FactoryResetConfirmationCode {
    fn new() -> Self {
        let secret = crate::auth::util::random_secret(FACTORY_RESET_CONFIRMATION_CODE_LENGTH);
        Self(SerializableSecretString::from(secret))
    }
}

impl PartialEq for FactoryResetConfirmationCode {
    fn eq(&self, other: &Self) -> bool {
        use secrecy::ExposeSecret as _;
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Validate for FactoryResetConfirmationCode {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.0.len() != FACTORY_RESET_CONFIRMATION_CODE_LENGTH {
            errors.add("confirmation", ValidationError::new("length").with_message(Cow::Owned(format!("Invalid confirmation code: Expected length is {FACTORY_RESET_CONFIRMATION_CODE_LENGTH}."))));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid confirmation code.")]
pub struct InvalidConfirmationCode;
