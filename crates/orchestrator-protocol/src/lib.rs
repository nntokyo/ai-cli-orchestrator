#![forbid(unsafe_code)]

//! Shared command/event contract between the Rust core and the TypeScript UI.
//!
//! This crate defines types only. It does not grant Tauri IPC permissions and
//! does not imply that a command is currently callable by the frontend.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Current wire-contract version.
pub const PROTOCOL_VERSION: u32 = 1;

/// Correlates a frontend request with a core response/event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(transparent)]
#[ts(export, export_to = "protocol/")]
pub struct RequestId(pub String);

/// Minimal request contract used to validate code generation before runtime
/// IPC is enabled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export, export_to = "protocol/")]
pub enum CoreRequest {
    GetBootstrapStatus { request_id: RequestId },
}

/// Core-to-frontend event contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export, export_to = "protocol/")]
pub enum CoreEvent {
    BootstrapStatus {
        request_id: RequestId,
        protocol_version: u32,
        status: String,
    },
    ProtocolError {
        request_id: Option<RequestId>,
        code: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_is_non_zero() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }
}
