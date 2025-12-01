// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use self::member::*;
pub use self::member_role::*;
pub use self::nickname::*;
pub use super::user_repository::UsersStats;

mod member {
    use serdev::Serialize;

    use crate::{members::MemberRole, models::BareJid};

    #[derive(Debug, Clone)]
    #[derive(Serialize)]
    #[cfg_attr(feature = "test", derive(serdev::Deserialize))]
    pub struct Member {
        pub jid: BareJid,
        pub role: Option<MemberRole>,
    }
}

mod member_role {
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
    #[derive(strum::EnumIter, strum::EnumString, strum::Display)]
    pub enum MemberRole {
        #[strum(serialize = "MEMBER")]
        Member,
        #[strum(serialize = "ADMIN")]
        Admin,
    }

    impl Default for MemberRole {
        fn default() -> Self {
            Self::Member
        }
    }

    impl PartialOrd for MemberRole {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            (*self as u8).partial_cmp(&(*other as u8))
        }
    }
}

mod nickname {
    use serdev::Serialize;
    use validator::{Validate, ValidationErrors};

    pub const NICKNAME_MAX_LENGTH: usize = 48;

    #[derive(Clone)]
    #[derive(Serialize, serdev::Deserialize)]
    #[serde(validate = "Validate::validate")]
    #[repr(transparent)]
    pub struct Nickname(String);

    impl Validate for Nickname {
        fn validate(&self) -> Result<(), ValidationErrors> {
            use std::borrow::Cow;

            use validator::{ValidateNonControlCharacter as _, ValidationError};

            let mut errors = ValidationErrors::new();

            if self.0.len() > NICKNAME_MAX_LENGTH {
                errors.add(
                    "__all__",
                    ValidationError::new("max_length").with_message(Cow::Owned(format!(
                        "Invalid nickname: Max length is {NICKNAME_MAX_LENGTH}."
                    ))),
                );
            }

            if !self.0.validate_non_control_character() {
                errors.add(
                    "__all__",
                    ValidationError::new("control_characters").with_message(Cow::Borrowed(
                        "Invalid nickname: Control characters prohibited.",
                    )),
                );
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }

    impl std::ops::Deref for Nickname {
        type Target = String;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl std::fmt::Display for Nickname {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Display::fmt(&self.0, f)
        }
    }

    impl std::fmt::Debug for Nickname {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&self.0, f)
        }
    }

    impl From<crate::xmpp::JidNode> for Nickname {
        fn from(value: crate::xmpp::JidNode) -> Self {
            // NOTE: No need to validate, `JidNode`s are parsed
            //   and already follow strict rules.
            Self(value.to_string())
        }
    }

    impl From<crate::models::EmailAddress> for Nickname {
        fn from(value: crate::models::EmailAddress) -> Self {
            // NOTE: No need to validate, `EmailAddress`es are parsed
            //   and already follow strict rules.
            Self(value.local_part().to_owned())
        }
    }

    #[cfg(feature = "test")]
    impl Nickname {
        pub fn from_string_unsafe(str: String) -> Self {
            Self(str)
        }
    }
}
