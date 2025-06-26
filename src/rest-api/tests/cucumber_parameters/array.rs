// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::{
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};

use cucumber::Parameter;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE: Regex = Regex::new(r"^([^, ]+)(?:, ([^, ])+)*(?: and ([^ ]+))?$").unwrap();
}

#[derive(Debug, Parameter)]
#[param(name = "array", regex = r"((?:[^, ]+)(?:, [^, ]+)*(?: and [^ ]+)?)")]
pub struct Array<P: Parameter>(pub Vec<P>);

impl<P: Parameter> Deref for Array<P> {
    type Target = Vec<P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P: Parameter + FromStr> FromStr for Array<P>
where
    <P as FromStr>::Err: Debug,
{
    type Err = <P as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = RE.captures(s).unwrap();
        let captures_iter = captures.iter().skip(1).flatten();
        let mut res = Vec::with_capacity(captures_iter.clone().count());
        for capture in captures_iter {
            res.push(P::from_str(capture.as_str())?);
        }
        Ok(Self(res))
    }
}

impl<P: Parameter + Display> Display for Array<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
