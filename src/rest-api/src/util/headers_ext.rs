// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub trait HeaderValueExt {
    fn starts_with(&self, prefix: &str) -> bool;
}

impl HeaderValueExt for axum::http::HeaderValue {
    fn starts_with(&self, prefix: &str) -> bool {
        self.as_bytes().starts_with(prefix.as_bytes())
    }
}
