#![forbid(unsafe_code)]

//! Provider-independent domain core for AI CLI Orchestrator.
//!
//! Tauri, webview, and provider-process integration must remain outside this
//! crate so the core can be unit-tested without launching the desktop shell.

pub mod identity;
pub mod session;

/// Product name shared by shell integrations.
pub const PRODUCT_NAME: &str = "AI CLI Orchestrator";

/// Minimal bootstrap probe used until the Task Execution Engine lands.
#[must_use]
pub const fn bootstrap_status() -> &'static str {
    "core-ready"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_probe_is_stable() {
        assert_eq!(bootstrap_status(), "core-ready");
        assert_eq!(PRODUCT_NAME, "AI CLI Orchestrator");
    }
}
