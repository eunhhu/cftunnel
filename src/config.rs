use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

impl IngressRule {
    pub fn is_catch_all(&self) -> bool {
        self.hostname.is_none()
    }

    pub fn get_port(&self) -> Option<u16> {
        if let Some(port_str) = self.service.rsplit(':').next() {
            port_str.parse().ok()
        } else {
            None
        }
    }

    pub fn get_protocol(&self) -> &str {
        if self.service.starts_with("http://") {
            "http"
        } else if self.service.starts_with("https://") {
            "https"
        } else if self.service.starts_with("ssh://") {
            "ssh"
        } else if self.service.starts_with("tcp://") {
            "tcp"
        } else if self.service.starts_with("udp://") {
            "udp"
        } else {
            "other"
        }
    }
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

impl CloudflaredConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;

        serde_yaml::from_str(&content)
            .with_context(|| "Failed to parse YAML")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        fs::write(path, yaml)?;
        Ok(())
    }

    pub fn get_ingress_rules(&self) -> Vec<&IngressRule> {
        self.ingress.iter().filter(|r| !r.is_catch_all()).collect()
    }

    pub fn add_rule(&mut self, hostname: String, service: String, no_tls_verify: bool) {
        let origin_request = if no_tls_verify {
            Some(OriginRequest {
                no_tls_verify: Some(true),
                http_host_header: None,
                extra: HashMap::new(),
            })
        } else {
            None
        };

        let rule = IngressRule {
            hostname: Some(hostname),
            service,
            origin_request,
        };

        // catch-all 앞에 삽입
        let pos = self.ingress.iter()
            .position(|r| r.is_catch_all())
            .unwrap_or(self.ingress.len());

        self.ingress.insert(pos, rule);
    }

    pub fn remove_rule(&mut self, index: usize) -> bool {
        let actual_rules: Vec<usize> = self.ingress.iter()
            .enumerate()
            .filter(|(_, r)| !r.is_catch_all())
            .map(|(i, _)| i)
            .collect();

        if index < actual_rules.len() {
            self.ingress.remove(actual_rules[index]);
            true
        } else {
            false
        }
    }

    pub fn update_rule(&mut self, index: usize, service: String) -> bool {
        let actual_rules: Vec<usize> = self.ingress.iter()
            .enumerate()
            .filter(|(_, r)| !r.is_catch_all())
            .map(|(i, _)| i)
            .collect();

        if index < actual_rules.len() {
            self.ingress[actual_rules[index]].service = service;
            true
        } else {
            false
        }
    }

    pub fn hostname_exists(&self, hostname: &str) -> bool {
        self.ingress.iter()
            .filter_map(|r| r.hostname.as_ref())
            .any(|h| h == hostname)
    }
}
