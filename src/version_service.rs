use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeVersion {
    pub version: String,
    pub date: String,
    pub lts: serde_json::Value, // Có thể là String (tên bản LTS) hoặc Boolean (false)
}

impl NodeVersion {
    pub fn is_lts(&self) -> bool {
        match &self.lts {
            serde_json::Value::Bool(b) => *b,
            serde_json::Value::String(_) => true,
            _ => false,
        }
    }

    pub fn lts_name(&self) -> String {
        match &self.lts {
            serde_json::Value::String(s) => s.clone(),
            _ => "No".to_string(),
        }
    }
}

pub fn fetch_node_versions() -> Result<Vec<NodeVersion>, Box<dyn Error>> {
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
