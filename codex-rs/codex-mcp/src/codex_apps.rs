//! Codex Apps support for the host-owned apps MCP server.
//!
//! This module owns the normalization that turns ChatGPT-hosted app
//! connector/tool metadata into model-visible MCP callable names.

use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use crate::mcp::CODEX_APPS_MCP_SERVER_NAME;
use crate::runtime::emit_duration;
use crate::tools::MCP_TOOLS_CACHE_WRITE_DURATION_METRIC;
use crate::tools::ToolInfo;
use anyhow::Context;
use codex_login::CodexAuth;
use codex_protocol::mcp::McpServerInfo;
use codex_utils_plugins::mcp_connector::is_connector_id_allowed;
use codex_utils_plugins::mcp_connector::sanitize_name;

pub(crate) fn normalize_codex_apps_tool_title(connector_name: Option<&str>, value: &str) -> String {
    let Some(connector_name) = connector_name
        .map(str::trim)
        .filter(|name| !name.is_empty())
    else {
        return value.to_string();
    };

    let prefix = format!("{connector_name}_");
    if let Some(stripped) = value.strip_prefix(&prefix)
        && !stripped.is_empty()
    {
        return stripped.to_string();
    }

    value.to_string()
}

pub(crate) fn normalize_codex_apps_callable_name(
    tool_name: &str,
    connector_id: Option<&str>,
    connector_name: Option<&str>,
) -> String {
    let tool_name = sanitize_name(tool_name);

    if let Some(connector_name) = connector_name
        .map(str::trim)
        .map(sanitize_name)
        .filter(|name| !name.is_empty())
        && let Some(stripped) = tool_name.strip_prefix(&connector_name)
        && !stripped.is_empty()
    {
        return stripped.to_string();
    }

    if let Some(connector_id) = connector_id
        .map(str::trim)
        .map(sanitize_name)
        .filter(|name| !name.is_empty())
        && let Some(stripped) = tool_name.strip_prefix(&connector_id)
        && !stripped.is_empty()
    {
        return stripped.to_string();
    }

    tool_name
}

pub(crate) fn normalize_codex_apps_callable_namespace(
    server_name: &str,
    connector_name: Option<&str>,
) -> String {
    if let Some(connector_name) = connector_name {
        format!("{}__{}", server_name, sanitize_name(connector_name))
    } else {
        server_name.to_string()
    }
}
