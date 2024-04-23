// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rocket::serde::json as serde_json;
use sea_orm::{entity::prelude::*, Set};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use self::model::NotificationPayload;
use self::model::{
    Template, NOTIFICATION_DATA_KEY_INVITATION_ACCEPT_LINK,
    NOTIFICATION_DATA_KEY_INVITATION_REJECT_LINK,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "notification")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub created_at: DateTimeUtc,
    template: Template,
    data: Json,
}

impl Model {
    fn string(&self, key: &'static str) -> String {
        self.data[key]
            .as_str()
            .expect(format!("'{key}' key not found in notification data").as_str())
            .to_string()
    }

    pub fn payload(&self) -> NotificationPayload {
        match self.template {
            Template::WorkspaceInvitation => NotificationPayload::WorkspaceInvitation {
                accept_link: self.string(NOTIFICATION_DATA_KEY_INVITATION_ACCEPT_LINK),
                reject_link: self.string(NOTIFICATION_DATA_KEY_INVITATION_REJECT_LINK),
            },
        }
    }
}

impl ActiveModel {
    pub fn set_content(&mut self, payload: &NotificationPayload) {
        self.template = Set(payload.template());
        self.data = Set(match payload {
            NotificationPayload::WorkspaceInvitation {
                accept_link,
                reject_link,
            } => json!({
                NOTIFICATION_DATA_KEY_INVITATION_ACCEPT_LINK: accept_link,
                NOTIFICATION_DATA_KEY_INVITATION_REJECT_LINK: reject_link,
            }),
        });
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

mod model {
    use std::{fmt::Display, str::FromStr};

    use sea_orm::{
        entity::prelude::*,
        sea_query::{ArrayType, ValueType, ValueTypeErr},
        TryGetError,
    };
    use serde::{Deserialize, Serialize};

    pub(super) const NOTIFICATION_DATA_KEY_INVITATION_ACCEPT_LINK: &'static str =
        "invitation_accept_link";
    pub(super) const NOTIFICATION_DATA_KEY_INVITATION_REJECT_LINK: &'static str =
        "invitation_reject_link";

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum NotificationPayload {
        WorkspaceInvitation {
            accept_link: String,
            reject_link: String,
        },
    }

    impl NotificationPayload {
        pub fn template(&self) -> Template {
            match self {
                Self::WorkspaceInvitation { .. } => Template::WorkspaceInvitation,
            }
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Template {
        WorkspaceInvitation,
    }

    impl Template {
        pub fn iterator() -> impl Iterator<Item = Self> {
            [Self::WorkspaceInvitation].iter().copied()
        }
    }

    impl Display for Template {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::WorkspaceInvitation => write!(f, "workspace_invitation"),
            }
        }
    }

    impl FromStr for Template {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "workspace_invitation" => Ok(Self::WorkspaceInvitation),
                _ => Err(format!("Unknown notification template: {s:?}")),
            }
        }
    }

    impl From<Template> for sea_orm::Value {
        fn from(value: Template) -> Self {
            Self::String(Some(Box::new(value.to_string())))
        }
    }

    impl ValueType for Template {
        fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
            match v {
                Value::String(Some(value)) => {
                    Self::from_str((*value).as_str()).map_err(|_| ValueTypeErr)
                }
                _ => Err(ValueTypeErr),
            }
        }

        fn type_name() -> String {
            stringify!(Template).to_string()
        }

        fn array_type() -> ArrayType {
            ArrayType::String
        }

        fn column_type() -> ColumnType {
            ColumnType::string(Some(
                Self::iterator().map(|v| v.to_string().len()).max().unwrap() as u32,
            ))
        }
    }

    impl sea_orm::TryGetable for Template {
        fn try_get_by<I: sea_orm::ColIdx>(
            res: &sea_orm::prelude::QueryResult,
            index: I,
        ) -> Result<Self, TryGetError> {
            let value: String = res.try_get_by(index).map_err(TryGetError::DbErr)?;
            Self::from_str(value.as_str())
                // Technically, the value is not `null`, but we wouldn't want to unsafely unwrap here.
                .map_err(|e| TryGetError::Null(format!("{:?}", e)))
        }
    }
}
