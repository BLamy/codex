#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod config;
#[cfg(not(target_arch = "wasm32"))]
mod events;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod metrics;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod provider;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod trace_context;

#[cfg(not(target_arch = "wasm32"))]
mod otlp;
#[cfg(not(target_arch = "wasm32"))]
mod targets;

#[cfg(not(target_arch = "wasm32"))]
use crate::metrics::Result as MetricsResult;
#[cfg(not(target_arch = "wasm32"))]
use serde::Serialize;
#[cfg(not(target_arch = "wasm32"))]
use strum_macros::Display;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::config::OtelExporter;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::config::OtelHttpProtocol;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::config::OtelSettings;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::config::OtelTlsConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::config::StatsigMetricsSettings;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::config::validate_span_attributes;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::events::session_telemetry::AuthEnvTelemetryMetadata;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::events::session_telemetry::SessionTelemetry;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::events::session_telemetry::SessionTelemetryMetadata;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::metrics::runtime_metrics::RuntimeMetricTotals;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::metrics::runtime_metrics::RuntimeMetricsSummary;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::metrics::timer::Timer;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::metrics::*;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::provider::OtelProvider;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::trace_context::context_from_w3c_trace_context;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::trace_context::current_span_trace_id;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::trace_context::current_span_w3c_trace_context;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::trace_context::set_parent_from_context;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::trace_context::set_parent_from_w3c_trace_context;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::trace_context::span_w3c_trace_context;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::trace_context::traceparent_context_from_env;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::trace_context::validate_tracestate_entries;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::trace_context::validate_tracestate_member;
#[cfg(not(target_arch = "wasm32"))]
pub use codex_utils_string::sanitize_metric_tag_value;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Serialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum ToolDecisionSource {
    AutomatedReviewer,
    Config,
    User,
}

/// Maps to API/auth `AuthMode` to avoid a circular dependency on codex-core.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum TelemetryAuthMode {
    ApiKey,
    Chatgpt,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<codex_app_server_protocol::AuthMode> for TelemetryAuthMode {
    fn from(mode: codex_app_server_protocol::AuthMode) -> Self {
        match mode {
            codex_app_server_protocol::AuthMode::ApiKey => Self::ApiKey,
            codex_app_server_protocol::AuthMode::Chatgpt
            | codex_app_server_protocol::AuthMode::ChatgptAuthTokens
            | codex_app_server_protocol::AuthMode::AgentIdentity => Self::Chatgpt,
        }
    }
}

/// Start a metrics timer using the globally installed metrics client.
#[cfg(not(target_arch = "wasm32"))]
pub fn start_global_timer(name: &str, tags: &[(&str, &str)]) -> MetricsResult<Timer> {
    let Some(metrics) = crate::metrics::global() else {
        return Err(MetricsError::ExporterDisabled);
    };
    metrics.start_timer(name, tags)
}

/// Returns the resolved Statsig metrics settings for the globally installed
/// OTEL metrics client, if the active metrics exporter is Statsig.
#[cfg(not(target_arch = "wasm32"))]
pub fn global_statsig_metrics_settings() -> Option<StatsigMetricsSettings> {
    crate::metrics::global_statsig_settings()
}
