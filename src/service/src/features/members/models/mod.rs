// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod member;
mod member_role;

pub use self::member::*;
pub use self::member_role::*;
pub use self::nickname::*;

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
