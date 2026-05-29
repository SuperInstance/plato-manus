use serde::Deserialize;
use std::path::PathBuf;

/// YAML-based access control policy for Manus operations.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct HandPolicy {
    #[serde(default)]
    pub allowed_paths: Vec<String>,
    #[serde(default)]
    pub denied_paths: Vec<String>,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default)]
    pub denied_domains: Vec<String>,
    #[serde(default)]
    pub allowed_devices: Vec<String>,
    #[serde(default)]
    pub denied_devices: Vec<String>,
}

impl HandPolicy {
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    pub fn permissive() -> Self {
        Self {
            allowed_paths: vec!["/tmp".into()],
            denied_paths: vec![],
            allowed_domains: vec![],
            denied_domains: vec![],
            allowed_devices: vec![],
            denied_devices: vec![],
        }
    }

    /// Check if a file path is allowed under this policy.
    pub fn is_path_allowed(&self, path: &str) -> bool {
        let p = PathBuf::from(path);
        // Denied takes precedence
        if self.denied_paths.iter().any(|d| p.starts_with(d)) {
            return false;
        }
        // If allowed_paths is empty, allow all non-denied
        if self.allowed_paths.is_empty() {
            return true;
        }
        self.allowed_paths.iter().any(|a| p.starts_with(a))
    }

    /// Check if a domain is allowed under this policy.
    pub fn is_domain_allowed(&self, url: &str) -> bool {
        let domain = extract_domain(url);
        if let Some(d) = &domain {
            if self.denied_domains.iter().any(|denied| d.ends_with(denied)) {
                return false;
            }
            if self.allowed_domains.is_empty() {
                return true;
            }
            return self.allowed_domains.iter().any(|allowed| d.ends_with(allowed));
        }
        false
    }

    /// Check if a device is allowed under this policy.
    pub fn is_device_allowed(&self, device: &str) -> bool {
        if self.denied_devices.iter().any(|d| device == d) {
            return false;
        }
        if self.allowed_devices.is_empty() {
            return true;
        }
        self.allowed_devices.iter().any(|a| device == a)
    }
}

fn extract_domain(url: &str) -> Option<String> {
    url.split("://")
        .nth(1)?
        .split('/')
        .next()
        .map(|s| s.split(':').next().unwrap_or(s).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissive_policy() {
        let p = HandPolicy::permissive();
        assert!(p.is_path_allowed("/tmp/test.txt"));
        assert!(p.is_domain_allowed("https://example.com/api"));
    }

    #[test]
    fn test_denied_path() {
        let p = HandPolicy {
            allowed_paths: vec!["/tmp".into()],
            denied_paths: vec!["/tmp/secret".into()],
            ..Default::default()
        };
        assert!(p.is_path_allowed("/tmp/hello.txt"));
        assert!(!p.is_path_allowed("/tmp/secret/key"));
    }

    #[test]
    fn test_domain_policy() {
        let p = HandPolicy {
            allowed_domains: vec!["example.com".into()],
            denied_domains: vec!["evil.example.com".into()],
            ..Default::default()
        };
        assert!(p.is_domain_allowed("https://example.com/api"));
        assert!(!p.is_domain_allowed("https://evil.example.com"));
        assert!(!p.is_domain_allowed("https://other.com"));
    }
}
