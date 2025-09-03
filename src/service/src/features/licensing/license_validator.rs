// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::SystemTime;

use anyhow::Context as _;
use biscuit::{builder_ext::AuthorizerExt, macros::authorizer, AuthorizerBuilder, Biscuit};
use hickory_proto::rr::domain::Name as DomainName;
use lazy_static::lazy_static;

use super::BiscuitExt as _;

const ALLOWED_PUBLIC_KEYS_BYTES: [[u8; 32]; 3] = [
    [
        153, 163, 126, 156, 132, 104, 100, 86, 106, 218, 109, 78, 216, 168, 71, 120, 159, 213, 185,
        77, 21, 91, 182, 9, 255, 212, 149, 176, 234, 215, 182, 79,
    ],
    [
        105, 204, 75, 135, 167, 2, 109, 93, 137, 171, 7, 170, 89, 56, 209, 244, 134, 251, 47, 111,
        229, 62, 230, 72, 64, 53, 100, 243, 192, 159, 225, 217,
    ],
    [
        163, 212, 137, 121, 114, 141, 64, 123, 237, 14, 50, 101, 65, 211, 253, 16, 184, 60, 184,
        207, 221, 148, 141, 110, 19, 125, 217, 205, 244, 35, 175, 80,
    ],
];

lazy_static! {
    static ref ALLOWED_PUBLIC_KEYS: [biscuit::PublicKey; 3] = {
        ALLOWED_PUBLIC_KEYS_BYTES.map(|bytes| {
            biscuit::PublicKey::from_bytes(&bytes, biscuit::Algorithm::Ed25519).unwrap()
        })
    };
}

#[derive(Debug, PartialEq, Eq)]
#[derive(thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid signature.")]
    InvalidSignature,
    #[error("Wrong domain.")]
    WrongDomain,
    #[error("License expired.")]
    Expired,
    #[error("Bad Pod API version.")]
    BadApiVersion,
    #[error("No user limit.")]
    NoUserLimit,
    #[error("Missing condition.")]
    MissingCondition,
    #[error("Internal error: {0}")]
    Internal(InternalError),
}

#[derive(Debug, PartialEq, Eq)]
#[derive(thiserror::Error)]
pub enum InternalError {
    #[error("Biscuit error: {0:?}")]
    Biscuit(#[from] biscuit::error::Token),
}

#[derive(Debug, PartialEq, Eq)]
pub enum FailedCheck {
    Domain,
    Expiry,
    ApiVersion,
    NoUserLimit,
    MissingCondition,
}

impl TryFrom<biscuit::error::Token> for FailedCheck {
    type Error = biscuit::error::Token;

    fn try_from(err: biscuit::error::Token) -> Result<Self, Self::Error> {
        use biscuit::error as e;

        match &err {
            e::Token::FailedLogic(e::Logic::Unauthorized { checks, .. }) => {
                for check in checks.into_iter() {
                    if let Ok(res) = Self::try_from(check.clone()) {
                        return Ok(res);
                    }
                }
                Err(err)
            }
            _ => Err(err),
        }
    }
}

impl TryFrom<biscuit::error::FailedCheck> for FailedCheck {
    type Error = biscuit::error::FailedCheck;

    fn try_from(err: biscuit::error::FailedCheck) -> Result<Self, Self::Error> {
        use biscuit::error as e;

        match &err {
            e::FailedCheck::Block(e::FailedBlockCheck { rule, .. }) => {
                if rule.contains("api_version") {
                    Ok(FailedCheck::ApiVersion)
                } else if rule.contains("expiry") {
                    Ok(FailedCheck::Expiry)
                } else if rule.contains("domain") {
                    Ok(FailedCheck::Domain)
                } else {
                    Err(err)
                }
            }
            e::FailedCheck::Authorizer(e::FailedAuthorizerCheck { rule, .. }) => {
                if rule.contains("user_limit") {
                    Ok(FailedCheck::NoUserLimit)
                } else if rule.contains("check if right") {
                    Ok(FailedCheck::MissingCondition)
                } else {
                    Err(err)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LicenseValidator {
    api_version: semver::Version,
    domain: DomainName,
}

impl LicenseValidator {
    pub fn new(domain: DomainName) -> Self {
        let api_version = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        Self {
            api_version,
            domain,
        }
    }

    /// WARNING: Any attempt to reverse-engineer the Prose Pod API
    ///   for the purpose of bypassing or removing the user limit,
    ///   is strictly prohibited and may expose you to legal action.
    fn authorizer(&self) -> AuthorizerBuilder {
        let now = SystemTime::now();
        authorizer!(
            r#"
            time({now});
            api_version({api_version});
            domain({domain});

            // Require a user limit to be present.
            check if user_limit($limit);

            // Require some condition to be present (to avoid reuse).
            check if right("domain", $domain)
                  or right("api_version", $version);
            "#,
            api_version = self.api_version.to_string(),
            domain = self.domain.to_string(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct License {
    biscuit: Biscuit,
    user_limit: u32,
    expiry: Option<SystemTime>,
}

impl License {
    pub fn to_bytes(&self) -> Result<Vec<u8>, anyhow::Error> {
        (self.biscuit.to_vec()).context("Could not serialize Biscuit to bytes")
    }
    pub fn id(&self) -> &[u8] {
        self.biscuit.container().authority.signature.to_bytes()
    }
}

impl License {
    pub fn deserialize(
        bytes: impl AsRef<[u8]>,
        validator: &LicenseValidator,
    ) -> Result<License, ValidationError> {
        use biscuit::error::{Format as FormatError, Token as TokenError};

        let mut last_err: Option<TokenError> = None;
        for pub_key in ALLOWED_PUBLIC_KEYS.iter() {
            match Biscuit::from(&bytes, pub_key) {
                Ok(biscuit) => return License::new(biscuit, validator),
                Err(err) => last_err = Some(err),
            };
        }
        match last_err.unwrap() {
            TokenError::Format(FormatError::Signature(_)) => Err(ValidationError::InvalidSignature),
            err => Err(ValidationError::Internal(InternalError::Biscuit(err))),
        }
    }

    /// Creates a new [`License`] after validating the given [`Biscuit`].
    pub fn new(biscuit: Biscuit, validator: &LicenseValidator) -> Result<Self, ValidationError> {
        let authorizer = validator
            .authorizer()
            // Validate the token if all checks passed.
            .allow_all();
        match dbg!(&biscuit).authorize(&dbg!(authorizer)) {
            Ok(_) => {}
            Err(err) => return Err(ValidationError::from(err)),
        };

        // Pre-read some values from the Biscuit to speed up future checks.
        let user_limit = Self::user_limit_(&biscuit)?;
        let expiry = Self::expiry_(&biscuit)?;

        Ok(Self {
            biscuit,
            user_limit,
            expiry,
        })
    }

    pub fn try_as_valid(&self, validator: &LicenseValidator) -> Result<&Self, ValidationError> {
        let authorizer = validator
            .authorizer()
            // Validate the token if all checks passed.
            .allow_all();
        match self.biscuit.authorize(&authorizer) {
            Ok(_) => Ok(self),
            Err(err) => Err(ValidationError::from(err)),
        }
    }

    pub fn is_valid(&self, validator: &LicenseValidator) -> bool {
        match self.try_as_valid(validator) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn authorize(
        &self,
        authorizer: AuthorizerBuilder,
        validator: &LicenseValidator,
    ) -> Result<(), ValidationError> {
        let ref authorizer = validator.authorizer().merge(authorizer);
        match self.biscuit.authorize(authorizer) {
            Ok(_) => Ok(()),
            Err(err) => Err(ValidationError::from(err)),
        }
    }

    pub fn user_limit(&self) -> u32 {
        self.user_limit
    }

    pub fn expiry(&self) -> Option<SystemTime> {
        self.expiry
    }

    pub fn allows_user_count(&self, user_count: u32) -> bool {
        self.user_limit >= user_count
    }
}

// MARK: - Helpers

impl License {
    fn user_limit_(biscuit: &Biscuit) -> Result<u32, ValidationError> {
        let mut authorizer = biscuit.authorizer()?;
        let res: Vec<(i64,)> = authorizer.query("data($limit) <- user_limit($limit)")?;
        match res.into_iter().next() {
            Some((limit,)) => Ok(limit.clamp(u32::MIN as i64, u32::MAX as i64) as u32),
            None => Err(ValidationError::NoUserLimit),
        }
    }

    fn expiry_(biscuit: &Biscuit) -> Result<Option<SystemTime>, ValidationError> {
        let mut authorizer = biscuit.authorizer()?;
        let res: Vec<(SystemTime,)> = authorizer.query("data($expiry) <- expiry($expiry)")?;
        match res.into_iter().next() {
            Some((limit,)) => Ok(Some(limit)),
            None => Ok(None),
        }
    }
}

// MARK: - Boilerplate

impl From<FailedCheck> for ValidationError {
    fn from(err: FailedCheck) -> Self {
        match err {
            FailedCheck::Domain => Self::WrongDomain,
            FailedCheck::Expiry => Self::Expired,
            FailedCheck::ApiVersion => Self::BadApiVersion,
            FailedCheck::NoUserLimit => Self::NoUserLimit,
            FailedCheck::MissingCondition => Self::MissingCondition,
        }
    }
}

impl From<biscuit::error::Token> for ValidationError {
    fn from(err: biscuit::error::Token) -> Self {
        match FailedCheck::try_from(err) {
            Ok(err) => Self::from(err),
            Err(err) => Self::Internal(InternalError::Biscuit(err)),
        }
    }
}

// MARK: - Debug helpers

#[cfg(feature = "test")]
#[allow(unused)]
impl License {
    /// Prints the Datalog source of the underlying Biscuit.
    pub(crate) fn source(&self) -> Result<String, InternalError> {
        let ref biscuit = self.biscuit;
        let mut res = biscuit.print_block_source(0)?;

        for n in 1..(biscuit.block_count()) {
            res.push_str("\n---\n");
            let block_source = biscuit.print_block_source(n)?;
            res.push_str(&block_source);
        }

        Ok(res)
    }
}
