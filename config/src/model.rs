use std::{fmt::Display, ops::Deref, str::FromStr};

use jid::NodePart;
use serde_with::DeserializeFromStr;

#[derive(Debug, Clone, DeserializeFromStr)]
#[repr(transparent)]
pub struct JidNode(NodePart);

impl Deref for JidNode {
    type Target = <NodePart as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl FromStr for JidNode {
    type Err = <NodePart as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NodePart::from_str(s).map(Self)
    }
}

impl Display for JidNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
