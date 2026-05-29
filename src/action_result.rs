use serde::{Deserialize, Serialize};

/// Generic result that agents can parse.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ActionResult {
    pub fn ok(message: impl Into<String>) -> Self {
        Self { success: true, message: message.into(), data: None, error: None }
    }

    pub fn ok_with_data(message: impl Into<String>, data: impl Into<String>) -> Self {
        Self { success: true, message: message.into(), data: Some(data.into()), error: None }
    }

    pub fn denied(message: impl Into<String>) -> Self {
        Self { success: false, message: "Policy denied".into(), data: None, error: Some(message.into()) }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self { success: false, message: message.into(), data: None, error: None }
    }
}

/// Directory listing as text entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextListing {
    pub path: String,
    pub entries: Vec<ListingEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListingEntry {
    pub name: String,
    pub kind: String, // "file" | "dir"
    pub size: u64,
}

/// File content as text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextContent {
    pub path: String,
    pub content: String,
}

/// HTTP response as text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextResponse {
    pub status: u16,
    pub body: String,
    pub url: String,
}

/// Device status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceStatus {
    pub device: String,
    pub state: DeviceState,
    pub info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceState {
    On,
    Off,
    Unknown,
}
