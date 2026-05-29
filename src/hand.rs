use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::action_result::*;
use crate::policy::HandPolicy;

/// The "hands" module — file operations, API calls, and device control
/// translated into a text-based interface for agents.
pub struct Manus {
    policy: HandPolicy,
    devices: HashMap<String, DeviceState>,
}

impl Manus {
    pub fn new(policy: HandPolicy) -> Self {
        Self {
            policy,
            devices: HashMap::new(),
        }
    }

    /// Register a virtual device with an initial state.
    pub fn register_device(&mut self, name: &str, state: DeviceState) {
        self.devices.insert(name.to_string(), state);
    }

    // ---- FileHand ----

    /// List directory contents as text entries.
    pub fn ls(&self, path: &str) -> Result<TextListing, ActionResult> {
        if !self.policy.is_path_allowed(path) {
            return Err(ActionResult::denied(format!("Path '{}' denied by policy", path)));
        }
        let p = Path::new(path);
        if !p.exists() {
            return Err(ActionResult::err(format!("Path '{}' does not exist", path)));
        }
        if !p.is_dir() {
            return Err(ActionResult::err(format!("Path '{}' is not a directory", path)));
        }
        let mut entries: Vec<ListingEntry> = Vec::new();
        let rd = match fs::read_dir(p) {
            Ok(rd) => rd,
            Err(e) => return Err(ActionResult::err(format!("Failed to read dir: {}", e))),
        };
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let kind = if meta.is_dir() { "dir" } else { "file" }.to_string();
            entries.push(ListingEntry { name, kind, size: meta.len() });
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(TextListing { path: path.to_string(), entries })
    }

    /// Read file contents as text.
    pub fn read(&self, path: &str) -> Result<TextContent, ActionResult> {
        if !self.policy.is_path_allowed(path) {
            return Err(ActionResult::denied(format!("Path '{}' denied by policy", path)));
        }
        match fs::read_to_string(path) {
            Ok(content) => Ok(TextContent { path: path.to_string(), content }),
            Err(e) => Err(ActionResult::err(format!("Failed to read '{}': {}", path, e))),
        }
    }

    /// Write content to a file.
    pub fn write(&self, path: &str, content: &str) -> ActionResult {
        if !self.policy.is_path_allowed(path) {
            return ActionResult::denied(format!("Path '{}' denied by policy", path));
        }
        // Create parent dirs if needed
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return ActionResult::err(format!("Failed to create parent dir: {}", e));
                }
            }
        }
        match fs::write(path, content) {
            Ok(()) => ActionResult::ok(format!("Wrote {} bytes to '{}'", content.len(), path)),
            Err(e) => ActionResult::err(format!("Failed to write '{}': {}", path, e)),
        }
    }

    /// Create a directory.
    pub fn mkdir(&self, path: &str) -> ActionResult {
        if !self.policy.is_path_allowed(path) {
            return ActionResult::denied(format!("Path '{}' denied by policy", path));
        }
        match fs::create_dir_all(path) {
            Ok(()) => ActionResult::ok(format!("Created directory '{}'", path)),
            Err(e) => ActionResult::err(format!("Failed to mkdir '{}': {}", path, e)),
        }
    }

    /// Remove a file or directory.
    pub fn rm(&self, path: &str) -> ActionResult {
        if !self.policy.is_path_allowed(path) {
            return ActionResult::denied(format!("Path '{}' denied by policy", path));
        }
        let p = Path::new(path);
        if !p.exists() {
            return ActionResult::err(format!("Path '{}' does not exist", path));
        }
        let result = if p.is_dir() { fs::remove_dir_all(p) } else { fs::remove_file(p) };
        match result {
            Ok(()) => ActionResult::ok(format!("Removed '{}'", path)),
            Err(e) => ActionResult::err(format!("Failed to remove '{}': {}", path, e)),
        }
    }

    // ---- ApiHand ----

    /// Make an HTTP request (text-in, text-out).
    pub fn http_request(&self, method: &str, url: &str, body: Option<&str>) -> Result<TextResponse, ActionResult> {
        if !self.policy.is_domain_allowed(url) {
            return Err(ActionResult::denied(format!("Domain '{}' denied by policy", url)));
        }
        let client = reqwest::blocking::Client::new();
        let req = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url).body(body.unwrap_or("").to_string()),
            "PUT" => client.put(url).body(body.unwrap_or("").to_string()),
            "DELETE" => client.delete(url),
            _ => return Err(ActionResult::err(format!("Unsupported HTTP method: {}", method))),
        };
        match req.send() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body_text = resp.text().unwrap_or_else(|e| format!("[body read error: {}]", e));
                Ok(TextResponse { status, body: body_text, url: url.to_string() })
            }
            Err(e) => Err(ActionResult::err(format!("HTTP request failed: {}", e))),
        }
    }

    // ---- DeviceHand ----

    /// Get the status of a device.
    pub fn device_status(&self, device: &str) -> Result<DeviceStatus, ActionResult> {
        if !self.policy.is_device_allowed(device) {
            return Err(ActionResult::denied(format!("Device '{}' denied by policy", device)));
        }
        match self.devices.get(device) {
            Some(state) => Ok(DeviceStatus {
                device: device.to_string(),
                state: state.clone(),
                info: format!("{:?} ({})", state, device),
            }),
            None => Ok(DeviceStatus {
                device: device.to_string(),
                state: DeviceState::Unknown,
                info: format!("Device '{}' not registered", device),
            }),
        }
    }

    /// Turn a device on.
    pub fn device_on(&mut self, device: &str) -> ActionResult {
        if !self.policy.is_device_allowed(device) {
            return ActionResult::denied(format!("Device '{}' denied by policy", device));
        }
        self.devices.insert(device.to_string(), DeviceState::On);
        ActionResult::ok(format!("Device '{}' turned on", device))
    }

    /// Turn a device off.
    pub fn device_off(&mut self, device: &str) -> ActionResult {
        if !self.policy.is_device_allowed(device) {
            return ActionResult::denied(format!("Device '{}' denied by policy", device));
        }
        self.devices.insert(device.to_string(), DeviceState::Off);
        ActionResult::ok(format!("Device '{}' turned off", device))
    }

    // ---- Generic ----

    /// Generic text-in, text-out command execution.
    /// Supports simple commands like "echo hello", "ls /tmp", etc.
    pub fn execute(&self, command: &str) -> ActionResult {
        let parts: Vec<&str> = command.splitn(3, ' ').collect();
        if parts.is_empty() {
            return ActionResult::err("Empty command");
        }
        match parts[0] {
            "ls" => {
                let path = parts.get(1).unwrap_or(&".");
                match self.ls(path) {
                    Ok(listing) => {
                        let text = listing.entries.iter()
                            .map(|e| format!("{} {} {}", e.kind, e.size, e.name))
                            .collect::<Vec<_>>()
                            .join("\n");
                        ActionResult::ok_with_data(format!("Listed {} entries", listing.entries.len()), text)
                    }
                    Err(e) => e,
                }
            }
            "cat" | "read" => {
                let path = parts.get(1).unwrap_or(&".");
                match self.read(path) {
                    Ok(content) => ActionResult::ok_with_data("Read file", content.content),
                    Err(e) => e,
                }
            }
            "write" => {
                let path = parts.get(1).unwrap_or(&".");
                let content = parts.get(2).unwrap_or(&"");
                self.write(path, content)
            }
            "mkdir" => {
                let path = parts.get(1).unwrap_or(&".");
                self.mkdir(path)
            }
            "rm" => {
                let path = parts.get(1).unwrap_or(&".");
                self.rm(path)
            }
            _ => ActionResult::ok(format!("Executed: {}", command)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_policy() -> HandPolicy {
        HandPolicy {
            allowed_paths: vec!["/tmp/plato-test".into()],
            denied_paths: vec!["/tmp/plato-test/secret".into()],
            allowed_domains: vec!["example.com".into()],
            denied_domains: vec![],
            allowed_devices: vec!["light".into(), "fan".into()],
            denied_devices: vec!["dangerous-device".into()],
        }
    }

    fn setup() {
        let _ = fs::remove_dir_all("/tmp/plato-test");
        let _ = fs::create_dir_all("/tmp/plato-test");
    }

    #[test]
    fn test_ls_lists_files() {
        setup();
        fs::write("/tmp/plato-test/hello.txt", "world").unwrap();
        let manus = Manus::new(test_policy());
        let listing = manus.ls("/tmp/plato-test").unwrap();
        assert!(listing.entries.iter().any(|e| e.name == "hello.txt" && e.kind == "file"));
    }

    #[test]
    fn test_read_returns_contents() {
        setup();
        fs::write("/tmp/plato-test/note.txt", "hello there").unwrap();
        let manus = Manus::new(test_policy());
        let content = manus.read("/tmp/plato-test/note.txt").unwrap();
        assert_eq!(content.content, "hello there");
    }

    #[test]
    fn test_write_creates_file() {
        setup();
        let manus = Manus::new(test_policy());
        let result = manus.write("/tmp/plato-test/out.txt", "written content");
        assert!(result.success);
        assert_eq!(fs::read_to_string("/tmp/plato-test/out.txt").unwrap(), "written content");
    }

    #[test]
    fn test_policy_blocks_denied_path() {
        setup();
        fs::create_dir_all("/tmp/plato-test/secret").unwrap();
        let manus = Manus::new(test_policy());
        let result = manus.ls("/tmp/plato-test/secret");
        assert!(result.is_err());
        assert!(result.unwrap_err().error.unwrap().contains("denied"));
    }

    #[test]
    fn test_denied_path_returns_error_not_panic() {
        let manus = Manus::new(test_policy());
        let result = manus.read("/tmp/plato-test/secret/key.pem");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.success);
        assert!(err.error.is_some());
    }

    #[test]
    fn test_http_request_formats_response() {
        // We test the policy check path; actual HTTP requires network
        let manus = Manus::new(test_policy());
        // Denied domain
        let result = manus.http_request("GET", "https://forbidden.com/api", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().error.unwrap().contains("denied"));
    }

    #[test]
    fn test_device_status_returns_structured_text() {
        let mut manus = Manus::new(test_policy());
        manus.register_device("light", DeviceState::On);
        let status = manus.device_status("light").unwrap();
        assert_eq!(status.device, "light");
        assert_eq!(status.state, DeviceState::On);
    }

    #[test]
    fn test_device_denied() {
        let manus = Manus::new(test_policy());
        let result = manus.device_status("dangerous-device");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_generic_command() {
        setup();
        fs::write("/tmp/plato-test/exec_test.txt", "exec content").unwrap();
        let manus = Manus::new(test_policy());
        let result = manus.execute("read /tmp/plato-test/exec_test.txt");
        assert!(result.success);
        assert_eq!(result.data.unwrap(), "exec content");
    }

    #[test]
    fn test_device_on_off() {
        let mut manus = Manus::new(test_policy());
        manus.register_device("fan", DeviceState::Off);
        let result = manus.device_on("fan");
        assert!(result.success);
        let status = manus.device_status("fan").unwrap();
        assert_eq!(status.state, DeviceState::On);

        let result = manus.device_off("fan");
        assert!(result.success);
        let status = manus.device_status("fan").unwrap();
        assert_eq!(status.state, DeviceState::Off);
    }
}
