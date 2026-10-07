use crate::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum LtsStatus {
    Bool(bool),
    Named(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeVersion {
    pub version: String,
    pub date: String,
    pub lts: LtsStatus,
}

impl NodeVersion {
    pub fn is_lts(&self) -> bool {
        matches!(&self.lts, LtsStatus::Named(_))
    }

    pub fn lts_name(&self) -> Option<&str> {
        match &self.lts {
            LtsStatus::Named(name) => Some(name),
            _ => None,
        }
    }
}

pub fn fetch_node_versions() -> anyhow::Result<Vec<NodeVersion>> {
    let res = ureq::get("https://nodejs.org/dist/index.json")
        .set("User-Agent", "nvm-rust-gui")
        .call()?;

    let versions: Vec<NodeVersion> = res.into_json()?;
    Ok(versions)
}

