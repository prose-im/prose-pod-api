// prose-pod-api
//
// Copyright: 2024–2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use iso8601_timestamp::Timestamp as ISOTimestamp;
use time::OffsetDateTime;

use crate::error::{self, Error};

#[derive(Debug)]
#[derive(serdev::Deserialize)]
#[repr(transparent)]
pub struct Timestamp(ISOTimestamp);

impl Timestamp {
    pub fn try_into_offset_date_time(self) -> Result<OffsetDateTime, Error> {
        OffsetDateTime::from_unix_timestamp(
            self.duration_since(ISOTimestamp::UNIX_EPOCH)
                .whole_milliseconds() as i64,
        )
        .map_err(|err| {
            Error::from(error::BadRequest {
                reason: format!("Timestamp out of `OffsetDateTime` bounds: {err}"),
            })
        })
    }
}

impl Deref for Timestamp {
    type Target = ISOTimestamp;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryInto<OffsetDateTime> for Timestamp {
    type Error = Error;

    fn try_into(self) -> Result<OffsetDateTime, Self::Error> {
        self.try_into_offset_date_time()
    }
}
