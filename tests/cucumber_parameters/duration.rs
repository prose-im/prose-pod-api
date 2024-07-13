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
    regex = r"\d+ (?:year|month|week|day)s?(?: \d+ (?:year|month|week|day)s?)*|infinite"
)]
pub enum Duration {
    Finite(String),
    Infinite,
}

impl FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "infinite" {
            return Ok(Self::Infinite);
        }

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
            _ => Ok(Self::Finite(value)),
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
            Self::Finite(d) => {
                PossiblyInfinite::Finite(ISODuration::parse(&d).unwrap().try_into().unwrap())
            }
            Self::Infinite => PossiblyInfinite::Infinite,
        }
    }
}
