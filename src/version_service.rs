use reqwest::blocking::Client;
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
    let client = Client::new();
    let res = client
        .get("https://nodejs.org/dist/index.json")
        .header("User-Agent", "nvm-rust-gui")
        .send()?;

    let versions: Vec<NodeVersion> = res.json()?;

    // Mặc định JSON từ Node.js đã sắp xếp từ mới đến cũ,
    // nhưng ta có thể đảm bảo lại nếu cần.
    // Ở đây ta giữ nguyên vì Node.js API trả về bản mới nhất ở đầu.

    Ok(versions)
}
