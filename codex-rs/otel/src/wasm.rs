use codex_protocol::protocol::W3cTraceContext;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::time::Duration;
use strum_macros::Display;
use thiserror::Error;

include!("metrics/names.rs");

pub fn sanitize_metric_tag_value(value: &str) -> String {
    codex_utils_string::sanitize_metric_tag_value(value)
}

pub fn validate_span_attributes(attributes: &BTreeMap<String, String>) -> std::io::Result<()> {
    if attributes.keys().any(String::is_empty) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "configured span attribute key must not be empty",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct OtelSettings {
    pub environment: String,
    pub service_name: String,
    pub service_version: String,
    pub codex_home: PathBuf,
    pub exporter: OtelExporter,
    pub trace_exporter: OtelExporter,
    pub metrics_exporter: OtelExporter,
    pub runtime_metrics: bool,
    pub span_attributes: BTreeMap<String, String>,
    pub tracestate: BTreeMap<String, BTreeMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatsigMetricsSettings {
    pub environment: String,
}

#[derive(Clone, Debug)]
pub enum OtelHttpProtocol {
    Binary,
    Json,
}

#[derive(Clone, Debug, Default)]
pub struct OtelTlsConfig {
    pub ca_certificate: Option<AbsolutePathBuf>,
    pub client_certificate: Option<AbsolutePathBuf>,
    pub client_private_key: Option<AbsolutePathBuf>,
}

#[derive(Clone, Debug)]
pub enum OtelExporter {
    None,
    Statsig,
    OtlpGrpc {
        endpoint: String,
        headers: HashMap<String, String>,
        tls: Option<OtelTlsConfig>,
    },
    OtlpHttp {
        endpoint: String,
        headers: HashMap<String, String>,
        protocol: OtelHttpProtocol,
        tls: Option<OtelTlsConfig>,
    },
}

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("metrics exporter is disabled")]
    ExporterDisabled,
    #[error("runtime metrics snapshot reader is not enabled")]
    RuntimeSnapshotUnavailable,
    #[error("invalid metrics configuration: {message}")]
    InvalidConfig { message: String },
}

pub type Result<T> = std::result::Result<T, MetricsError>;
pub type MetricsResult<T> = Result<T>;

pub fn record_process_start_once(_metrics: &MetricsClient, _originator: &str) -> Result<bool> {
    Ok(false)
}

#[derive(Clone, Debug)]
pub enum MetricsExporter {
    Otlp(OtelExporter),
    InMemory,
}

#[derive(Clone, Debug)]
pub struct MetricsConfig {
    pub environment: String,
    pub service_name: String,
    pub service_version: String,
    pub exporter: MetricsExporter,
    pub export_interval: Option<Duration>,
    pub runtime_reader: bool,
    pub default_tags: BTreeMap<String, String>,
}

impl MetricsConfig {
    pub fn otlp(
        environment: impl Into<String>,
        service_name: impl Into<String>,
        service_version: impl Into<String>,
        exporter: OtelExporter,
    ) -> Self {
        Self {
            environment: environment.into(),
            service_name: service_name.into(),
            service_version: service_version.into(),
            exporter: MetricsExporter::Otlp(exporter),
            export_interval: None,
            runtime_reader: false,
            default_tags: BTreeMap::new(),
        }
    }

    pub fn with_export_interval(mut self, interval: Duration) -> Self {
        self.export_interval = Some(interval);
        self
    }

    pub fn with_runtime_reader(mut self) -> Self {
        self.runtime_reader = true;
        self
    }

    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Result<Self> {
        self.default_tags.insert(key.into(), value.into());
        Ok(self)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MetricsClient;

impl MetricsClient {
    pub fn new(_config: MetricsConfig) -> Result<Self> {
        Ok(Self)
    }

    pub fn counter(&self, _name: &str, _inc: i64, _tags: &[(&str, &str)]) -> Result<()> {
        Ok(())
    }

    pub fn histogram(&self, _name: &str, _value: i64, _tags: &[(&str, &str)]) -> Result<()> {
        Ok(())
    }

    pub fn record_duration(
        &self,
        _name: &str,
        _duration: Duration,
        _tags: &[(&str, &str)],
    ) -> Result<()> {
        Ok(())
    }

    pub fn start_timer(&self, name: &str, tags: &[(&str, &str)]) -> Result<Timer> {
        Ok(Timer::new(name, tags))
    }

    pub fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Timer {
    name: String,
    tags: Vec<(String, String)>,
}

impl Timer {
    fn new(name: &str, tags: &[(&str, &str)]) -> Self {
        Self {
            name: name.to_string(),
            tags: tags
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        }
    }

    pub fn record(&self, _additional_tags: &[(&str, &str)]) -> Result<()> {
        let _ = (&self.name, &self.tags);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RuntimeMetricTotals {
    pub count: u64,
    pub duration_ms: u64,
}

impl RuntimeMetricTotals {
    pub fn is_empty(self) -> bool {
        self.count == 0 && self.duration_ms == 0
    }

    pub fn merge(&mut self, other: Self) {
        self.count = self.count.saturating_add(other.count);
        self.duration_ms = self.duration_ms.saturating_add(other.duration_ms);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RuntimeMetricsSummary {
    pub tool_calls: RuntimeMetricTotals,
    pub api_calls: RuntimeMetricTotals,
    pub streaming_events: RuntimeMetricTotals,
    pub websocket_calls: RuntimeMetricTotals,
    pub websocket_events: RuntimeMetricTotals,
    pub responses_api_overhead_ms: u64,
    pub responses_api_inference_time_ms: u64,
    pub responses_api_engine_iapi_ttft_ms: u64,
    pub responses_api_engine_service_ttft_ms: u64,
    pub responses_api_engine_iapi_tbt_ms: u64,
    pub responses_api_engine_service_tbt_ms: u64,
    pub turn_ttft_ms: u64,
    pub turn_ttfm_ms: u64,
}

impl RuntimeMetricsSummary {
    pub fn is_empty(self) -> bool {
        self == Self::default()
    }

    pub fn merge(&mut self, other: Self) {
        self.tool_calls.merge(other.tool_calls);
        self.api_calls.merge(other.api_calls);
        self.streaming_events.merge(other.streaming_events);
        self.websocket_calls.merge(other.websocket_calls);
        self.websocket_events.merge(other.websocket_events);
    }

    pub fn responses_api_summary(&self) -> RuntimeMetricsSummary {
        *self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuthEnvTelemetryMetadata {
    pub openai_api_key_env_present: bool,
    pub codex_api_key_env_present: bool,
    pub codex_api_key_env_enabled: bool,
    pub provider_env_key_name: Option<String>,
    pub provider_env_key_present: Option<bool>,
    pub refresh_token_url_override_present: bool,
}

#[derive(Debug, Clone)]
pub struct SessionTelemetryMetadata {
    pub auth_mode: Option<String>,
    pub auth_env: AuthEnvTelemetryMetadata,
    pub account_id: Option<String>,
    pub account_email: Option<String>,
    pub originator: String,
    pub service_name: Option<String>,
    pub session_source: String,
    pub model: String,
    pub slug: String,
    pub log_user_prompts: bool,
    pub app_version: &'static str,
    pub terminal_type: String,
}

#[derive(Debug, Clone)]
pub struct SessionTelemetry {
    pub metadata: SessionTelemetryMetadata,
}

impl SessionTelemetry {
    pub fn new<ThreadId, SessionSource>(
        _conversation_id: ThreadId,
        model: &str,
        slug: &str,
        account_id: Option<String>,
        account_email: Option<String>,
        auth_mode: Option<TelemetryAuthMode>,
        originator: String,
        log_user_prompts: bool,
        terminal_type: String,
        session_source: SessionSource,
    ) -> SessionTelemetry
    where
        SessionSource: ToString,
    {
        Self {
            metadata: SessionTelemetryMetadata {
                auth_mode: auth_mode.map(|mode| mode.to_string()),
                auth_env: AuthEnvTelemetryMetadata::default(),
                account_id,
                account_email,
                originator: sanitize_metric_tag_value(originator.as_str()),
                service_name: None,
                session_source: session_source.to_string(),
                model: model.to_string(),
                slug: slug.to_string(),
                log_user_prompts,
                app_version: env!("CARGO_PKG_VERSION"),
                terminal_type,
            },
        }
    }

    pub fn with_auth_env(mut self, auth_env: AuthEnvTelemetryMetadata) -> Self {
        self.metadata.auth_env = auth_env;
        self
    }

    pub fn with_model(mut self, model: &str, slug: &str) -> Self {
        self.metadata.model = model.to_string();
        self.metadata.slug = slug.to_string();
        self
    }

    pub fn with_metrics_service_name(mut self, service_name: &str) -> Self {
        self.metadata.service_name = Some(sanitize_metric_tag_value(service_name));
        self
    }

    pub fn with_metrics(self, _metrics: MetricsClient) -> Self {
        self
    }

    pub fn with_metrics_without_metadata_tags(self, _metrics: MetricsClient) -> Self {
        self
    }

    pub fn with_metrics_config(self, _config: MetricsConfig) -> Result<Self> {
        Ok(self)
    }

    pub fn with_provider_metrics(self, _provider: &OtelProvider) -> Self {
        self
    }

    pub fn counter(&self, _name: &str, _inc: i64, _tags: &[(&str, &str)]) {}

    pub fn histogram(&self, _name: &str, _value: i64, _tags: &[(&str, &str)]) {}

    pub fn record_duration(&self, _name: &str, _duration: Duration, _tags: &[(&str, &str)]) {}

    pub fn record_startup_phase(
        &self,
        _phase: &'static str,
        _duration: Duration,
        _status: Option<&'static str>,
    ) {
    }

    pub fn record_turn_ttft(&self, _duration: Duration) {}

    pub fn record_plugin_install_elicitation_sent(
        &self,
        _tool_type: &str,
        _tool_id: &str,
        _tool_name: &str,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_plugin_install_suggestion(
        &self,
        _tool_type: &str,
        _tool_id: &str,
        _tool_name: &str,
        _response_action: &str,
        _user_confirmed: bool,
        _completed: bool,
    ) {
    }

    pub fn record_responses<T>(&self, _handle_responses_span: &tracing::Span, _event: &T) {}

    #[allow(clippy::too_many_arguments)]
    pub fn conversation_starts<ReasoningEffort, ReasoningSummary, ApprovalPolicy, SandboxPolicy>(
        &self,
        _provider_name: &str,
        _reasoning_effort: Option<ReasoningEffort>,
        _reasoning_summary: ReasoningSummary,
        _context_window: Option<i64>,
        _auto_compact_token_limit: Option<i64>,
        _approval_policy: ApprovalPolicy,
        _sandbox_policy: SandboxPolicy,
        _mcp_servers: Vec<&str>,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_api_request(
        &self,
        _attempt: u64,
        _status: Option<u16>,
        _error: Option<&str>,
        _duration: Duration,
        _auth_header_attached: bool,
        _auth_header_name: Option<&str>,
        _retry_after_unauthorized: bool,
        _recovery_mode: Option<&str>,
        _recovery_phase: Option<&str>,
        _endpoint: &str,
        _request_id: Option<&str>,
        _cf_ray: Option<&str>,
        _auth_error: Option<&str>,
        _auth_error_code: Option<&str>,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_websocket_connect(
        &self,
        _duration: Duration,
        _status: Option<u16>,
        _error: Option<&str>,
        _auth_header_attached: bool,
        _auth_header_name: Option<&str>,
        _retry_after_unauthorized: bool,
        _recovery_mode: Option<&str>,
        _recovery_phase: Option<&str>,
        _endpoint: &str,
        _connection_reused: bool,
        _request_id: Option<&str>,
        _cf_ray: Option<&str>,
        _auth_error: Option<&str>,
        _auth_error_code: Option<&str>,
    ) {
    }

    pub fn record_websocket_request(
        &self,
        _duration: Duration,
        _error: Option<&str>,
        _connection_reused: bool,
    ) {
    }

    pub fn record_auth_recovery(
        &self,
        _mode: &str,
        _step: &str,
        _outcome: &str,
        _request_id: Option<&str>,
        _cf_ray: Option<&str>,
        _auth_error: Option<&str>,
        _auth_error_code: Option<&str>,
        _recovery_reason: Option<&str>,
        _auth_state_changed: Option<bool>,
    ) {
    }

    pub fn record_websocket_event<T>(&self, _result: &T, _duration: Duration) {}

    pub fn log_sse_event<T>(&self, _response: &T, _duration: Duration) {}

    pub fn see_event_completed_failed<T>(&self, _error: &T)
    where
        T: std::fmt::Display,
    {
    }

    pub fn sse_event_completed(
        &self,
        _input_token_count: i64,
        _output_token_count: i64,
        _cached_token_count: Option<i64>,
        _reasoning_token_count: Option<i64>,
        _tool_token_count: i64,
    ) {
    }

    pub fn user_prompt<T>(&self, _items: &[T]) {}

    pub fn tool_decision<T>(
        &self,
        _tool_name: &str,
        _call_id: &str,
        _decision: &T,
        _source: ToolDecisionSource,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn log_tool_result_with_tags<F, Fut, E>(
        &self,
        _tool_name: &str,
        _call_id: &str,
        _arguments: &str,
        _extra_tags: &[(&str, &str)],
        _extra_trace_fields: &[(&str, &str)],
        f: F,
    ) -> std::result::Result<(String, bool), E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = std::result::Result<(String, bool), E>>,
        E: std::fmt::Display,
    {
        f().await
    }

    pub fn log_tool_failed(&self, _tool_name: &str, _error: &str) {}

    #[allow(clippy::too_many_arguments)]
    pub fn tool_result_with_tags(
        &self,
        _tool_name: &str,
        _call_id: &str,
        _arguments: &str,
        _duration: Duration,
        _success: bool,
        _output: &str,
        _extra_tags: &[(&str, &str)],
        _extra_trace_fields: &[(&str, &str)],
    ) {
    }

    pub fn start_timer(&self, name: &str, tags: &[(&str, &str)]) -> Result<Timer> {
        Ok(Timer::new(name, tags))
    }

    pub fn shutdown_metrics(&self) -> Result<()> {
        Ok(())
    }

    pub fn reset_runtime_metrics(&self) {}

    pub fn runtime_metrics_summary(&self) -> Option<RuntimeMetricsSummary> {
        None
    }
}

#[derive(Clone, Debug, Default)]
pub struct OtelProvider;

impl OtelProvider {
    pub fn shutdown(&self) {}

    pub fn from(
        _settings: &OtelSettings,
    ) -> std::result::Result<Option<Self>, Box<dyn std::error::Error>> {
        Ok(None)
    }

    pub fn metrics(&self) -> Option<&MetricsClient> {
        None
    }

    pub fn codex_export_filter(_meta: &tracing::Metadata<'_>) -> bool {
        false
    }

    pub fn log_export_filter(_meta: &tracing::Metadata<'_>) -> bool {
        false
    }

    pub fn trace_export_filter(_meta: &tracing::Metadata<'_>) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum ToolDecisionSource {
    AutomatedReviewer,
    Config,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum TelemetryAuthMode {
    ApiKey,
    Chatgpt,
}

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

pub fn start_global_timer(_name: &str, _tags: &[(&str, &str)]) -> Result<Timer> {
    Err(MetricsError::ExporterDisabled)
}

pub fn global() -> Option<MetricsClient> {
    None
}

pub fn global_statsig_metrics_settings() -> Option<StatsigMetricsSettings> {
    None
}

pub fn current_span_w3c_trace_context() -> Option<W3cTraceContext> {
    None
}

pub fn span_w3c_trace_context(_span: &tracing::Span) -> Option<W3cTraceContext> {
    None
}

pub fn current_span_trace_id() -> Option<String> {
    None
}

pub fn context_from_w3c_trace_context(_trace: &W3cTraceContext) -> Option<()> {
    None
}

pub fn set_parent_from_w3c_trace_context(_span: &tracing::Span, _trace: &W3cTraceContext) -> bool {
    false
}

pub fn set_parent_from_context<T>(_span: &tracing::Span, _context: T) {}

pub fn traceparent_context_from_env() -> Option<()> {
    None
}

pub fn validate_tracestate_entries(
    _entries: &BTreeMap<String, BTreeMap<String, String>>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn validate_tracestate_member(
    _key: &str,
    _fields: &BTreeMap<String, String>,
) -> std::result::Result<(), String> {
    Ok(())
}
