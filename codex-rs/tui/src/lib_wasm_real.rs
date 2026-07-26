extern crate self as codex_realtime_webrtc;
extern crate self as codex_config;
extern crate self as codex_connectors;
extern crate self as codex_app_server_client;
extern crate self as codex_core_plugins;
extern crate self as codex_core_skills;
extern crate self as codex_feedback;
extern crate self as codex_features;
extern crate self as codex_file_search;
extern crate self as codex_git_utils;
extern crate self as codex_model_provider_info;
extern crate self as codex_models_manager;
extern crate self as codex_message_history;
extern crate self as codex_otel;
extern crate self as codex_plugin;
extern crate self as codex_shell_command;
extern crate self as codex_utils_cli;
extern crate self as codex_utils_plugins;
extern crate self as codex_utils_sandbox_summary;
extern crate self as codex_utils_sleep_inhibitor;

use std::sync::Arc;
use std::sync::atomic::AtomicU16;
use std::sync::mpsc;

pub use codex_app_server_protocol::AppInfo;
pub use codex_app_server_protocol::ConfigLayerSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealtimeWebrtcEvent {
    Connected,
    LocalAudioLevel(u16),
    Closed,
    Failed(String),
}

#[derive(Debug)]
pub enum RealtimeWebrtcError {
    UnsupportedPlatform,
}

impl std::fmt::Display for RealtimeWebrtcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(f, "realtime WebRTC is not supported on wasm"),
        }
    }
}

pub type Result<T> = std::result::Result<T, RealtimeWebrtcError>;

pub struct StartedRealtimeWebrtcSession {
    pub offer_sdp: String,
    pub handle: RealtimeWebrtcSessionHandle,
    pub events: mpsc::Receiver<RealtimeWebrtcEvent>,
}

pub struct RealtimeWebrtcSessionHandle {
    local_audio_peak: Arc<AtomicU16>,
}

impl std::fmt::Debug for RealtimeWebrtcSessionHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealtimeWebrtcSessionHandle")
            .finish_non_exhaustive()
    }
}

pub struct RealtimeWebrtcSession;

impl RealtimeWebrtcSession {
    pub fn start() -> Result<StartedRealtimeWebrtcSession> {
        Err(RealtimeWebrtcError::UnsupportedPlatform)
    }
}

impl RealtimeWebrtcSessionHandle {
    pub fn apply_answer_sdp(&self, _answer_sdp: String) -> Result<()> {
        Err(RealtimeWebrtcError::UnsupportedPlatform)
    }

    pub fn close(&self) {}

    pub fn local_audio_peak(&self) -> Arc<AtomicU16> {
        self.local_audio_peak.clone()
    }
}

#[path = "legacy_core_wasm.rs"]
pub(crate) mod legacy_core;

#[path = "codex_connectors_wasm.rs"]
mod codex_connectors_wasm;

pub mod metadata {
    pub use crate::codex_connectors_wasm::metadata::*;
}

#[path = "codex_core_skills_wasm.rs"]
mod codex_core_skills_wasm;

pub mod model {
    pub use crate::codex_core_skills_wasm::model::*;
}

pub const OPENAI_CURATED_MARKETPLACE_NAME: &str = "openai-curated";
pub const OPENAI_API_CURATED_MARKETPLACE_NAME: &str = "openai-api-curated";

pub fn is_openai_curated_marketplace_name(marketplace_name: &str) -> bool {
    marketplace_name == OPENAI_CURATED_MARKETPLACE_NAME
        || marketplace_name == OPENAI_API_CURATED_MARKETPLACE_NAME
}

pub mod remote {
    pub const REMOTE_GLOBAL_MARKETPLACE_NAME: &str = "openai-curated-remote";
    pub const REMOTE_WORKSPACE_MARKETPLACE_NAME: &str = "workspace-directory";
    pub const REMOTE_WORKSPACE_SHARED_WITH_ME_MARKETPLACE_NAME: &str =
        "workspace-shared-with-me";
    pub const REMOTE_WORKSPACE_SHARED_WITH_ME_PRIVATE_MARKETPLACE_NAME: &str =
        "workspace-shared-with-me-private";
    pub const REMOTE_WORKSPACE_SHARED_WITH_ME_UNLISTED_MARKETPLACE_NAME: &str =
        "workspace-shared-with-me-unlisted";
}

#[path = "codex_app_server_client_wasm.rs"]
mod codex_app_server_client_wasm;

pub use codex_app_server_client_wasm::AppServerRequestHandle;
pub use codex_app_server_client_wasm::RemoteAppServerEndpoint;
pub use codex_app_server_client_wasm::TypedRequestError;

#[path = "codex_utils_sleep_inhibitor_wasm.rs"]
mod codex_utils_sleep_inhibitor_wasm;

pub use codex_utils_sleep_inhibitor_wasm::SleepInhibitor;

#[path = "codex_config_wasm.rs"]
mod codex_config_wasm;

pub use codex_config_wasm::CONFIG_TOML_FILE;
pub use codex_config_wasm::ConfigLayerEntry;
pub use codex_config_wasm::ConfigLayerStack;
pub use codex_config_wasm::ConfigLayerStackOrdering;
pub use codex_config_wasm::Constrained;
pub use codex_config_wasm::ConstraintError;
pub use codex_config_wasm::ConstraintResult;
pub use codex_config_wasm::ResidencyRequirement;
pub use codex_config_wasm::format_config_layer_source;

pub mod config_toml {
    pub use crate::codex_config_wasm::config_toml::*;
}

pub mod types {
    pub use crate::codex_config_wasm::types::*;
}

#[path = "codex_feedback_wasm.rs"]
mod codex_feedback_wasm;

pub use codex_feedback_wasm::CodexFeedback;
pub use codex_feedback_wasm::CODEX_APP_DIRECTORY_CACHE_ATTACHMENT_FILENAME;
pub use codex_feedback_wasm::CODEX_APPS_TOOLS_CACHE_ATTACHMENT_FILENAME;
pub use codex_feedback_wasm::DOCTOR_REPORT_ATTACHMENT_FILENAME;
pub use codex_feedback_wasm::FEEDBACK_DIAGNOSTICS_ATTACHMENT_FILENAME;
pub use codex_feedback_wasm::FeedbackDiagnostic;
pub use codex_feedback_wasm::FeedbackDiagnostics;
pub use codex_feedback_wasm::FeedbackSnapshot;
pub use codex_feedback_wasm::WINDOWS_SANDBOX_LOG_ATTACHMENT_FILENAME;

#[path = "codex_features_wasm.rs"]
mod codex_features_wasm;

pub use codex_features_wasm::FEATURES;
pub use codex_features_wasm::Feature;
pub use codex_features_wasm::FeatureSpec;
pub use codex_features_wasm::Features;
pub use codex_features_wasm::FeaturesToml;
pub use codex_features_wasm::Stage;

#[path = "codex_git_utils_wasm.rs"]
mod codex_git_utils_wasm;

pub use codex_git_utils_wasm::CommitLogEntry;
pub use codex_git_utils_wasm::current_branch_name;
pub use codex_git_utils_wasm::get_git_repo_root;
pub use codex_git_utils_wasm::local_git_branches;
pub use codex_git_utils_wasm::recent_commits;

/// Browser-safe subset of the current git-utils fsmonitor policy API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FsmonitorOverride {
    Disabled,
    BuiltIn,
}

impl FsmonitorOverride {
    pub const fn git_config_arg(self) -> &'static str {
        match self {
            Self::Disabled => "core.fsmonitor=false",
            Self::BuiltIn => "core.fsmonitor=true",
        }
    }
}

pub trait FsmonitorProbeRunner: Send {
    fn run_probe(
        &mut self,
        args: &[&str],
    ) -> impl std::future::Future<Output = Option<Vec<u8>>> + Send;
}

pub async fn detect_fsmonitor_override(
    _runner: &mut impl FsmonitorProbeRunner,
) -> FsmonitorOverride {
    // The browser command bridge cannot start Git's native fsmonitor daemon.
    FsmonitorOverride::Disabled
}

/// Browser-local representation of the current message-history cursor.
///
/// Persistent history remains owned by the app-server; the TUI only needs the
/// opaque absolute offset while it routes lookup events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryBatchCursor {
    end_offset: usize,
}

impl HistoryBatchCursor {
    pub fn new(end_offset: usize) -> Self {
        Self { end_offset }
    }

    pub fn end_offset(self) -> usize {
        self.end_offset
    }
}

#[path = "codex_model_provider_info_wasm.rs"]
mod codex_model_provider_info_wasm;

pub use codex_model_provider_info_wasm::AMAZON_BEDROCK_PROVIDER_ID;
pub use codex_model_provider_info_wasm::CHATGPT_CODEX_BASE_URL;
pub use codex_model_provider_info_wasm::ModelProviderAwsAuthInfo;
pub use codex_model_provider_info_wasm::ModelProviderInfo;
pub use codex_model_provider_info_wasm::OPENAI_PROVIDER_ID;
pub use codex_model_provider_info_wasm::WireApi;
pub use codex_model_provider_info_wasm::built_in_model_providers;

#[path = "codex_utils_cli_wasm.rs"]
mod codex_utils_cli_wasm;

pub use codex_utils_cli_wasm::format_env_display;
pub use codex_utils_cli_wasm::resume_hint;

#[path = "codex_utils_plugins_wasm.rs"]
mod codex_utils_plugins_wasm;

pub mod mention_syntax {
    pub use crate::codex_utils_plugins_wasm::mention_syntax::*;
}

#[path = "codex_file_search_wasm.rs"]
mod codex_file_search_wasm;

pub use codex_file_search_wasm::FileMatch;
pub use codex_file_search_wasm::FileSearchOptions;
pub use codex_file_search_wasm::FileSearchSession;
pub use codex_file_search_wasm::FileSearchSnapshot;
pub use codex_file_search_wasm::MatchType;
pub use codex_file_search_wasm::SessionReporter;
pub use codex_file_search_wasm::create_session;

#[path = "codex_otel_wasm.rs"]
mod codex_otel_wasm;

pub use codex_otel_wasm::AuthEnvTelemetryMetadata;
pub use codex_otel_wasm::RuntimeMetricTotals;
pub use codex_otel_wasm::RuntimeMetricsSummary;
pub use codex_otel_wasm::SessionTelemetry;
pub use codex_otel_wasm::TelemetryAuthMode;

#[path = "codex_plugin_wasm.rs"]
mod codex_plugin_wasm;

pub use codex_plugin_wasm::AppConnectorId;
pub use codex_plugin_wasm::PluginCapabilitySummary;

#[path = "codex_shell_command_wasm.rs"]
mod codex_shell_command_wasm;

pub mod bash {
    pub use crate::codex_shell_command_wasm::bash::*;
}

pub mod parse_command {
    pub use crate::codex_shell_command_wasm::parse_command::*;
}

pub mod collaboration_mode_presets {
    use codex_protocol::config_types::CollaborationModeMask;
    use codex_protocol::config_types::ModeKind;

    pub fn builtin_collaboration_mode_presets() -> Vec<CollaborationModeMask> {
        [ModeKind::Default, ModeKind::Plan]
            .into_iter()
            .map(|mode| CollaborationModeMask {
                name: mode.display_name().to_string(),
                mode: Some(mode),
                model: None,
                reasoning_effort: None,
                developer_instructions: None,
            })
            .collect()
    }
}

pub fn summarize_permission_profile(
    permission_profile: &codex_protocol::models::PermissionProfile,
    _cwd: &codex_utils_absolute_path::AbsolutePathBuf,
    _workspace_roots: &[codex_utils_absolute_path::AbsolutePathBuf],
) -> String {
    use codex_protocol::models::PermissionProfile;
    use codex_protocol::permissions::FileSystemSandboxKind;

    match permission_profile {
        PermissionProfile::Disabled => "danger-full-access".to_string(),
        PermissionProfile::External { .. } => "custom permissions (external sandbox)".to_string(),
        PermissionProfile::Managed { file_system, network } => {
            let file_system = file_system.to_sandbox_policy();
            let base = match file_system.kind {
                FileSystemSandboxKind::Unrestricted => "danger-full-access",
                FileSystemSandboxKind::Restricted if file_system.entries.is_empty() => "read-only",
                FileSystemSandboxKind::Restricted => "workspace-write",
                FileSystemSandboxKind::ExternalSandbox => "custom permissions",
            };
            if network.is_enabled() {
                format!("{base} (network access enabled)")
            } else {
                base.to_string()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AppServerTarget {
    Embedded,
    LocalDaemon { endpoint: RemoteAppServerEndpoint },
    Remote { endpoint: RemoteAppServerEndpoint },
}

mod app {
    pub(crate) mod app_server_requests {
        use codex_app_server_protocol::RequestId as AppServerRequestId;

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub(crate) enum ResolvedAppServerRequest {
            ExecApproval { id: String },
            FileChangeApproval { id: String },
            PermissionsApproval { id: String },
            UserInput { call_id: String },
            McpElicitation {
                server_name: String,
                request_id: AppServerRequestId,
            },
        }
    }
}

mod app_server_session {
    use crate::session_state::ThreadSessionState;
    use codex_app_server_protocol::Turn;

    #[derive(Debug)]
    pub(crate) struct AppServerStartedThread {
        pub(crate) session: ThreadSessionState,
        pub(crate) turns: Vec<Turn>,
    }
}

mod onboarding {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    pub(crate) fn mark_url_hyperlink(_buf: &mut Buffer, _area: Rect, _url: &str) {}
    pub(crate) fn mark_underlined_hyperlink(_buf: &mut Buffer, _area: Rect, _url: &str) {}
}

mod additional_dirs;
#[path = "app_backtrack_wasm.rs"]
mod app_backtrack;
mod app_command;
mod app_event;
mod app_event_sender;
mod app_server_approval_conversions;
mod approval_events;
mod ascii_animation;
mod auto_review_denials;
mod bottom_pane;
mod branch_summary;
mod chatwidget;
#[path = "clipboard_copy_wasm.rs"]
mod clipboard_copy;
mod clipboard_paste_wasm;
use clipboard_paste_wasm as clipboard_paste;
mod collaboration_modes;
mod color;
mod cwd_prompt;
pub(crate) mod custom_terminal;
#[path = "debug_config_wasm.rs"]
mod debug_config;
mod diff_model;
mod diff_render;
mod exec_cell;
mod exec_command;
#[path = "external_editor_wasm.rs"]
mod external_editor;
mod file_search;
mod frames;
mod get_git_diff;
mod git_action_directives;
mod goal_display;
#[path = "goal_files_wasm.rs"]
mod goal_files;
mod history_cell;
mod hooks_rpc;
mod ide_context;
#[path = "inline_visualization_wasm.rs"]
mod inline_visualization;
pub(crate) mod insert_history;
pub use insert_history::insert_history_lines;
mod key_hint;
mod keymap;
mod keymap_setup;
mod line_truncation;
pub(crate) mod live_wrap;
pub use live_wrap::RowBuilder;
mod markdown;
mod markdown_render;
mod markdown_stream;
mod markdown_text_merge;
mod mention_codec;
mod model_catalog;
mod motion;
mod multi_agents;
mod notifications;
mod pager_overlay;
mod permission_compat;
pub(crate) mod public_widgets;
mod render;
mod resize_reflow_cap;
mod selection_list;
mod service_tier_resolution;
mod session_log;
mod session_state;
mod shimmer;
mod skills_helpers;
mod slash_command;
mod status;
mod status_indicator_widget;
mod streaming;
mod style;
mod terminal_hyperlinks;
mod terminal_palette;
mod terminal_probe;
mod terminal_title;
mod table_detect;
mod text_formatting;
mod theme_picker;
mod token_usage;
mod tooltips;
mod transcript_reflow;
mod tui_wasm;
use tui_wasm as tui;
mod ui_consts;
pub(crate) mod update_action;
pub use update_action::UpdateAction;
mod version;
mod width;
mod wrapping;
mod workspace_command;
mod workspace_messages;
mod pets;
