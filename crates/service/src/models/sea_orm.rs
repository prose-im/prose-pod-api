// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(sea_orm::FromJsonQueryResult)]
#[repr(transparent)]
pub struct LinkedStringSet(pub linked_hash_set::LinkedHashSet<String>);

impl std::ops::Deref for LinkedStringSet {
    type Target = linked_hash_set::LinkedHashSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<linked_hash_set::LinkedHashSet<String>> for LinkedStringSet {
    fn from(value: linked_hash_set::LinkedHashSet<String>) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for LinkedStringSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let len = self.len();
        for (i, s) in self.0.iter().enumerate() {
            write!(f, "{s:?}")?;
            if i < len - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")
    }
}
