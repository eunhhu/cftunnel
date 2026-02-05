use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OriginRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_tls_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_host_header: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    pub service: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_request: Option<OriginRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudflaredConfig {
    pub tunnel: String,
    #[serde(rename = "credentials-file")]
    pub credentials_file: String,
    pub ingress: Vec<IngressRule>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or("/etc/cloudflared/config.yml".to_string());
    let content = fs::read_to_string(&path).expect("Failed to read file");

    match serde_yaml::from_str::<CloudflaredConfig>(&content) {
        Ok(config) => {
            println!("✓ Parse successful!");
            println!("  Tunnel: {}", config.tunnel);
            println!("  Rules: {}", config.ingress.len());
            for (i, rule) in config.ingress.iter().enumerate() {
                let host = rule.hostname.as_deref().unwrap_or("(catch-all)");
                println!("    {}: {} -> {}", i + 1, host, rule.service);
            }
        }
        Err(e) => {
            eprintln!("✗ Parse failed: {}", e);
        }
    }
}
