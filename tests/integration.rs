//! Integration tests for plato-manus

use plato_manus::*;
use std::fs;

fn permissive_policy() -> HandPolicy {
    HandPolicy::permissive()
}

fn restricted_policy() -> HandPolicy {
    HandPolicy {
        allowed_paths: vec!["/tmp/manus-test".into()],
        denied_paths: vec!["/tmp/manus-test/secret".into()],
        allowed_domains: vec!["example.com".into()],
        denied_domains: vec!["evil.com".into()],
        allowed_devices: vec!["light".into()],
        denied_devices: vec!["laser".into()],
    }
}

fn setup_test_dir() {
    let _ = fs::remove_dir_all("/tmp/manus-test");
    let _ = fs::create_dir_all("/tmp/manus-test");
}

#[test]
fn test_action_result_ok() {
    let r = ActionResult::ok("success");
    assert!(r.success);
    assert_eq!(r.message, "success");
    assert!(r.data.is_none());
    assert!(r.error.is_none());
}

#[test]
fn test_action_result_with_data() {
    let r = ActionResult::ok_with_data("done", "payload");
    assert!(r.success);
    assert_eq!(r.data.unwrap(), "payload");
}

#[test]
fn test_action_result_denied() {
    let r = ActionResult::denied("forbidden");
    assert!(!r.success);
    assert!(r.error.unwrap().contains("forbidden"));
}

#[test]
fn test_write_and_read_file() {
    setup_test_dir();
    let manus = Manus::new(permissive_policy());
    let result = manus.write("/tmp/manus-test/hello.txt", "hello world");
    assert!(result.success);

    let content = manus.read("/tmp/manus-test/hello.txt").unwrap();
    assert_eq!(content.content, "hello world");
}

#[test]
fn test_list_directory() {
    let dir = "/tmp/manus-ls-test";
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).unwrap();
    fs::write(format!("{}/a.txt", dir), "a").unwrap();
    fs::write(format!("{}/b.txt", dir), "b").unwrap();
    let manus = Manus::new(permissive_policy());
    let listing = manus.ls(dir).unwrap();
    assert_eq!(listing.entries.len(), 2);
    assert!(listing.entries.iter().all(|e| e.kind == "file"));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_mkdir_and_rm() {
    setup_test_dir();
    let manus = Manus::new(permissive_policy());
    let result = manus.mkdir("/tmp/manus-test/subdir");
    assert!(result.success);
    assert!(std::path::Path::new("/tmp/manus-test/subdir").is_dir());

    let result = manus.rm("/tmp/manus-test/subdir");
    assert!(result.success);
    assert!(!std::path::Path::new("/tmp/manus-test/subdir").exists());
}

#[test]
fn test_policy_blocks_denied_path() {
    setup_test_dir();
    fs::create_dir_all("/tmp/manus-test/secret").unwrap();
    let manus = Manus::new(restricted_policy());
    let result = manus.ls("/tmp/manus-test/secret");
    assert!(result.is_err());
}

#[test]
fn test_device_registration_and_control() {
    let mut manus = Manus::new(restricted_policy());
    manus.register_device("light", DeviceState::Off);

    let status = manus.device_status("light").unwrap();
    assert_eq!(status.state, DeviceState::Off);

    manus.device_on("light");
    let status = manus.device_status("light").unwrap();
    assert_eq!(status.state, DeviceState::On);

    manus.device_off("light");
    let status = manus.device_status("light").unwrap();
    assert_eq!(status.state, DeviceState::Off);
}

#[test]
fn test_denied_device_blocked() {
    let manus = Manus::new(restricted_policy());
    let result = manus.device_status("laser");
    assert!(result.is_err());
}

#[test]
fn test_execute_command_routing() {
    setup_test_dir();
    fs::write("/tmp/manus-test/exec.txt", "test content").unwrap();
    let manus = Manus::new(restricted_policy());
    let result = manus.execute("read /tmp/manus-test/exec.txt");
    assert!(result.success);
    assert_eq!(result.data.unwrap(), "test content");
}

#[test]
fn test_hand_policy_yaml_parsing() {
    let yaml = r#"
allowed_paths:
  - /tmp
denied_paths: []
allowed_domains:
  - example.com
denied_domains: []
allowed_devices: []
denied_devices: []
"#;
    let policy = HandPolicy::from_yaml(yaml).unwrap();
    assert!(policy.is_path_allowed("/tmp/test"));
    assert!(policy.is_domain_allowed("https://example.com/api"));
}
