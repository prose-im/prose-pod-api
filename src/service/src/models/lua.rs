// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Lua(pub String);

impl std::ops::Deref for Lua {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
