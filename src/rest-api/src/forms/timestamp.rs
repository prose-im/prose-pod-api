// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use chrono::{DateTime, Utc};
use iso8601_timestamp::Timestamp as ISOTimestamp;

use crate::error::{self, Error};

#[derive(Debug)]
#[derive(serdev::Deserialize)]
#[repr(transparent)]
pub struct Timestamp(ISOTimestamp);

impl Timestamp {
    pub fn try_into_chrono_datetime(self) -> Result<DateTime<Utc>, Error> {
        DateTime::from_timestamp_millis(
            self.duration_since(ISOTimestamp::UNIX_EPOCH)
                .whole_milliseconds() as i64,
        )
        .ok_or(Error::from(error::BadRequest {
            reason: "Timestamp out of `DateTime<Utc>` bounds.".to_string(),
        }))
    }
}

impl Deref for Timestamp {
    type Target = ISOTimestamp;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryInto<DateTime<Utc>> for Timestamp {
    type Error = Error;

    fn try_into(self) -> Result<DateTime<Utc>, Self::Error> {
        self.try_into_chrono_datetime()
    }
}
