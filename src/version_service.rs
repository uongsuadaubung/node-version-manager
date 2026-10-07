use crate::anyhow;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum LtsStatus {
    #[allow(dead_code)]
    Bool(bool),
    Named(String),
}

#[derive(Debug, Deserialize, Clone)]
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
    let bytes = crate::fetch::fetch_bytes("https://nodejs.org/dist/index.json")?;
    let versions: Vec<NodeVersion> = serde_json::from_slice(&bytes)?;
    Ok(versions)
}

