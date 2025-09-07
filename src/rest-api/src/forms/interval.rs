// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Debug)]
#[derive(serdev::Deserialize)]
pub struct Interval {
    pub interval: Option<iso8601_duration::Duration>,
}
