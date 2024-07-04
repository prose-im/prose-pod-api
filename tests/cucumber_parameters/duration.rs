// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use cucumber::codegen::Regex;
use cucumber::Parameter;
use iso8601_duration::Duration as ISODuration;
use service::deprecated::{DateLike, PossiblyInfinite};

#[derive(Debug, Parameter)]
#[param(
    name = "duration",
    regex = r"\d+ (?:year|month|week|day)s?(?: \d+ (?:year|month|week|day)s?)*"
)]
pub struct Duration(String);

impl FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let patterns = vec![
            (r"(\d+) years?", 'Y'),
            (r"(\d+) months?", 'M'),
            (r"(\d+) weeks?", 'W'),
            (r"(\d+) days?", 'D'),
        ];

        let mut value = "P".to_string();
        for (pattern, designator) in patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(s) {
                value.push_str(captures.get(1).unwrap().as_str());
                value.push(designator);
            }
        }

        match value.as_str() {
            "P" => Err(format!("Invalid `Duration`: {s}")),
            _ => Ok(Self(value)),
        }
    }
}

impl ToString for Duration {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Into<service::deprecated::Duration<DateLike>> for Duration {
    fn into(self) -> service::deprecated::Duration<DateLike> {
        ISODuration::parse(&self.0).unwrap().try_into().unwrap()
    }
}

impl Into<PossiblyInfinite<service::deprecated::Duration<DateLike>>> for Duration {
    fn into(self) -> PossiblyInfinite<service::deprecated::Duration<DateLike>> {
        PossiblyInfinite::Finite(self.into())
    }
}
