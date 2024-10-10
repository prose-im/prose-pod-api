// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{fmt::Display, str::FromStr};

use cucumber::codegen::Regex;
use cucumber::Parameter;
use iso8601_duration::Duration as ISODuration;
use service::model::{DateLike, PossiblyInfinite};

#[derive(Debug, Parameter)]
#[param(
    name = "duration",
    regex = r"\d+ (?:year|month|week|day|hour|minute|second)s?(?: \d+ (?:year|month|week|day|hour|minute|second)s?)*|infinite"
)]
pub enum Duration {
    Finite(ISODuration),
    Infinite,
}

impl Duration {
    pub fn seconds(&self) -> u32 {
        match self {
            Self::Finite(duration) => duration
                .num_seconds()
                .expect("Could not get seconds from duration.")
                as u32,
            Self::Infinite => panic!("Could not get seconds from infinite duration."),
        }
    }
}

impl FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "infinite" {
            return Ok(Self::Infinite);
        }

        let date_patterns = vec![
            (r"(\d+) years?", 'Y'),
            (r"(\d+) months?", 'M'),
            (r"(\d+) weeks?", 'W'),
            (r"(\d+) days?", 'D'),
        ];
        let time_patterns = vec![
            (r"(\d+) hours?", 'H'),
            (r"(\d+) minutes?", 'M'),
            (r"(\d+) seconds?", 'S'),
        ];

        let mut value = "P".to_string();
        for (pattern, designator) in date_patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(s) {
                value.push_str(captures.get(1).unwrap().as_str());
                value.push(designator);
            }
        }

        let mut has_time = false;
        for (pattern, designator) in time_patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(s) {
                if !has_time {
                    value.push('T');
                    has_time = true;
                }
                value.push_str(captures.get(1).unwrap().as_str());
                value.push(designator);
            }
        }

        if value.as_str() == "P" {
            return Err(format!("Invalid `Duration`: '{s}'"));
        }

        match ISODuration::parse(value.as_str()) {
            Ok(duration) => Ok(Self::Finite(duration)),
            Err(err) => Err(format!("Invalid `Duration`: {err:?}")),
        }
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Finite(d) => Display::fmt(d, f),
            Self::Infinite => write!(f, "infinite"),
        }
    }
}

impl Into<PossiblyInfinite<service::model::Duration<DateLike>>> for Duration {
    fn into(self) -> PossiblyInfinite<service::model::Duration<DateLike>> {
        match self {
            Self::Finite(d) => PossiblyInfinite::Finite(d.try_into().unwrap()),
            Self::Infinite => PossiblyInfinite::Infinite,
        }
    }
}
