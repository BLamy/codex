pub const DEFAULT_AGENTS_MD_FILENAME: &str = "AGENTS.md";
pub const LOCAL_AGENTS_MD_FILENAME: &str = ".codex/AGENTS.md";

pub fn check_execpolicy_for_warnings(
    _config: &config::Config,
    _warnings: &mut Vec<String>,
) {
}

pub fn format_exec_policy_error_with_source(err: impl std::fmt::Display) -> String {
    err.to_string()
}

pub fn grant_read_root_non_elevated(
    profile: codex_protocol::models::PermissionProfile,
    _path: std::path::PathBuf,
) -> codex_protocol::models::PermissionProfile {
    profile
}

pub fn web_search_detail(_config: &config::Config) -> codex_protocol::config_types::WebSearchMode {
    codex_protocol::config_types::WebSearchMode::default()
}

pub mod util {
    pub fn normalize_thread_name(name: &str) -> Option<String> {
        let normalized = name.trim();
        (!normalized.is_empty()).then(|| normalized.to_string())
    }
}

pub mod windows_sandbox {
    pub const ELEVATED_SANDBOX_NUX_ENABLED: bool = false;

    pub trait WindowsSandboxLevelExt {}

    pub fn sandbox_setup_is_complete(_codex_home: &std::path::Path) -> bool {
        false
    }
}

pub mod config {
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::Arc;

    pub use codex_config::Constrained;
    pub use codex_config::ConstraintError;
    pub use codex_config::ConstraintResult;
    use codex_config::ConfigLayerStack;
    use codex_config::config_toml::ConfigLockfileToml;
    use codex_config::config_toml::ProjectConfig;
    use codex_config::config_toml::RealtimeAudioConfig;
    use codex_config::config_toml::RealtimeConfig;
    use codex_config::types::ApprovalsReviewer;
    use codex_config::types::AuthCredentialsStoreMode;
    use codex_config::types::History;
    use codex_config::types::McpServerConfig;
    use codex_config::types::MemoriesConfig;
    use codex_config::types::ModelAvailabilityNuxConfig;
    use codex_config::types::Notice;
    use codex_config::types::OAuthCredentialsStoreMode;
    use codex_config::types::OtelConfig;
    use codex_config::types::SessionPickerViewMode;
    use codex_config::types::ToolSuggestConfig;
    use codex_config::types::TuiKeymap;
    use codex_config::types::TuiNotificationSettings;
    use codex_config::types::TuiPetAnchor;
    use codex_config::types::UriBasedFileOpener;
    use codex_config::types::WindowsSandboxModeToml;
    use codex_features::Feature;
    use codex_features::Features;
    use codex_model_provider_info::ModelProviderInfo;
    use codex_model_provider_info::OPENAI_PROVIDER_ID;
    use codex_model_provider_info::built_in_model_providers;
    use codex_protocol::config_types::AltScreenMode;
    use codex_protocol::config_types::ApprovalsReviewer as CoreApprovalsReviewer;
    use codex_protocol::config_types::AutoCompactTokenLimitScope;
    use codex_protocol::config_types::ForcedLoginMethod;
    use codex_protocol::config_types::Personality;
    use codex_protocol::config_types::ReasoningSummary;
    use codex_protocol::config_types::SERVICE_TIER_DEFAULT_REQUEST_VALUE;
    use codex_protocol::config_types::SandboxMode;
    use codex_protocol::config_types::ServiceTier;
    use codex_protocol::config_types::ShellEnvironmentPolicy;
    use codex_protocol::config_types::Verbosity;
    use codex_protocol::config_types::WebSearchConfig;
    use codex_protocol::config_types::WebSearchMode;
    use codex_protocol::models::ActivePermissionProfile;
    use codex_protocol::models::PermissionProfile;
    use codex_protocol::openai_models::ModelsResponse;
    use codex_protocol::openai_models::ReasoningEffort;
    use codex_protocol::permissions::FileSystemSandboxPolicy;
    use codex_protocol::permissions::NetworkSandboxPolicy;
    use codex_protocol::protocol::AskForApproval;
    use codex_protocol::protocol::SandboxPolicy;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use serde::Serialize;
    use toml::Value as TomlValue;

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
    pub struct CodeModeConfig {
        pub excluded_tool_namespaces: Vec<String>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct AgentRoleConfig {
        pub description: Option<String>,
        pub config_file: Option<PathBuf>,
        pub nickname_candidates: Option<Vec<String>>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CustomPermissionProfileSummary {
        pub id: String,
        pub description: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub enum ThreadStoreConfig {
        #[default]
        Local,
        InMemory { id: String },
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
    pub struct MultiAgentV2Config {
        pub max_concurrent_threads_per_session: usize,
        pub min_wait_timeout_ms: i64,
        pub max_wait_timeout_ms: i64,
        pub default_wait_timeout_ms: i64,
        pub usage_hint_enabled: bool,
        pub usage_hint_text: Option<String>,
        pub root_agent_usage_hint_text: Option<String>,
        pub subagent_usage_hint_text: Option<String>,
        pub tool_namespace: Option<String>,
        pub hide_spawn_agent_metadata: bool,
        pub non_code_mode_only: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum TerminalResizeReflowMaxRows {
        #[default]
        Auto,
        Disabled,
        Limit(usize),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct TerminalResizeReflowConfig {
        pub max_rows: TerminalResizeReflowMaxRows,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct GhostSnapshotConfig {
        pub ignore_large_untracked_files: Option<i64>,
        pub ignore_large_untracked_dirs: Option<i64>,
        pub disable_warnings: bool,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NetworkProxySpec;

    impl NetworkProxySpec {
        pub fn socks_enabled(&self) -> bool {
            false
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PermissionProfileSnapshot {
        permission_profile: PermissionProfile,
        active_permission_profile: Option<ActivePermissionProfile>,
        profile_workspace_roots: Vec<AbsolutePathBuf>,
    }

    impl PermissionProfileSnapshot {
        pub fn legacy(permission_profile: PermissionProfile) -> Self {
            Self {
                permission_profile,
                active_permission_profile: None,
                profile_workspace_roots: Vec::new(),
            }
        }

        pub fn active(
            permission_profile: PermissionProfile,
            active_permission_profile: ActivePermissionProfile,
        ) -> Self {
            Self::active_with_profile_workspace_roots(
                permission_profile,
                active_permission_profile,
                Vec::new(),
            )
        }

        pub fn active_with_profile_workspace_roots(
            permission_profile: PermissionProfile,
            active_permission_profile: ActivePermissionProfile,
            profile_workspace_roots: Vec<AbsolutePathBuf>,
        ) -> Self {
            Self {
                permission_profile,
                active_permission_profile: Some(active_permission_profile),
                profile_workspace_roots,
            }
        }

        pub fn from_session_snapshot(
            permission_profile: PermissionProfile,
            active_permission_profile: Option<ActivePermissionProfile>,
        ) -> Self {
            Self {
                permission_profile,
                active_permission_profile,
                profile_workspace_roots: Vec::new(),
            }
        }

        pub fn permission_profile(&self) -> &PermissionProfile {
            &self.permission_profile
        }

        pub fn active_permission_profile(&self) -> Option<ActivePermissionProfile> {
            self.active_permission_profile.clone()
        }

        pub fn profile_workspace_roots(&self) -> &[AbsolutePathBuf] {
            &self.profile_workspace_roots
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Permissions {
        pub approval_policy: Constrained<AskForApproval>,
        permission_profile: PermissionProfile,
        active_permission_profile: Option<ActivePermissionProfile>,
        workspace_roots: Vec<AbsolutePathBuf>,
        pub network: Option<NetworkProxySpec>,
        pub allow_login_shell: bool,
        pub shell_environment_policy: ShellEnvironmentPolicy,
        pub windows_sandbox_mode: Option<WindowsSandboxModeToml>,
        pub windows_sandbox_private_desktop: bool,
    }

    impl Default for Permissions {
        fn default() -> Self {
            Self {
                approval_policy: Constrained::allow_any(AskForApproval::default()),
                permission_profile: PermissionProfile::default(),
                active_permission_profile: None,
                workspace_roots: Vec::new(),
                network: None,
                allow_login_shell: true,
                shell_environment_policy: ShellEnvironmentPolicy::default(),
                windows_sandbox_mode: None,
                windows_sandbox_private_desktop: true,
            }
        }
    }

    impl From<PermissionProfile> for Permissions {
        fn from(permission_profile: PermissionProfile) -> Self {
            Self {
                permission_profile,
                ..Self::default()
            }
        }
    }

    impl Permissions {
        pub fn from_approval_and_profile(
            approval_policy: Constrained<AskForApproval>,
            permission_profile: Constrained<PermissionProfile>,
        ) -> ConstraintResult<Self> {
            Ok(Self {
                approval_policy,
                permission_profile: permission_profile.get().clone(),
                ..Self::default()
            })
        }

        pub fn set_workspace_roots(&mut self, workspace_roots: Vec<AbsolutePathBuf>) {
            self.workspace_roots = workspace_roots;
        }

        pub fn workspace_roots(&self) -> &[AbsolutePathBuf] {
            &self.workspace_roots
        }

        pub fn user_visible_workspace_roots(&self) -> &[AbsolutePathBuf] {
            &self.workspace_roots
        }

        pub fn profile_workspace_roots(&self) -> &[AbsolutePathBuf] {
            &[]
        }

        pub fn permission_profile(&self) -> &PermissionProfile {
            &self.permission_profile
        }

        pub fn can_set_permission_profile(
            &self,
            _permission_profile: &PermissionProfile,
        ) -> ConstraintResult<()> {
            Ok(())
        }

        pub fn set_permission_profile(
            &mut self,
            permission_profile: PermissionProfile,
        ) -> ConstraintResult<()> {
            self.permission_profile = permission_profile;
            self.active_permission_profile = None;
            Ok(())
        }

        pub fn set_permission_profile_from_session_snapshot(
            &mut self,
            snapshot: PermissionProfileSnapshot,
        ) -> ConstraintResult<()> {
            self.permission_profile = snapshot.permission_profile;
            self.active_permission_profile = snapshot.active_permission_profile;
            self.workspace_roots = snapshot.profile_workspace_roots;
            Ok(())
        }

        pub fn replace_permission_profile_from_session_snapshot(
            &mut self,
            snapshot: PermissionProfileSnapshot,
        ) -> ConstraintResult<()> {
            self.set_permission_profile_from_session_snapshot(snapshot)
        }

        pub fn effective_permission_profile(&self) -> PermissionProfile {
            self.permission_profile
                .clone()
                .materialize_project_roots_with_workspace_roots(&self.workspace_roots)
        }

        pub fn active_permission_profile(&self) -> Option<ActivePermissionProfile> {
            self.active_permission_profile.clone()
        }

        pub fn file_system_sandbox_policy(&self) -> FileSystemSandboxPolicy {
            self.effective_permission_profile()
                .file_system_sandbox_policy()
        }

        pub fn network_sandbox_policy(&self) -> NetworkSandboxPolicy {
            self.permission_profile.network_sandbox_policy()
        }

        pub fn legacy_sandbox_policy(&self, _cwd: &Path) -> SandboxPolicy {
            SandboxPolicy::DangerFullAccess
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ManagedFeatures {
        value: Features,
    }

    impl Default for ManagedFeatures {
        fn default() -> Self {
            Self {
                value: Features::default(),
            }
        }
    }

    impl ManagedFeatures {
        pub fn get(&self) -> &Features {
            &self.value
        }

        pub fn enabled(&self, feature: Feature) -> bool {
            self.value.enabled(feature)
        }

        pub fn set_enabled(&mut self, feature: Feature, enabled: bool) -> ConstraintResult<()> {
            self.value.set_enabled(feature, enabled);
            Ok(())
        }

        pub fn can_set(&self, _candidate: &Features) -> ConstraintResult<()> {
            Ok(())
        }
    }

    impl std::ops::Deref for ManagedFeatures {
        type Target = Features;

        fn deref(&self) -> &Self::Target {
            self.get()
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Config {
        pub config_layer_stack: ConfigLayerStack,
        pub startup_warnings: Vec<String>,
        pub model: Option<String>,
        pub service_tier: Option<String>,
        pub review_model: Option<String>,
        pub model_context_window: Option<i64>,
        pub model_auto_compact_token_limit: Option<i64>,
        pub model_auto_compact_token_limit_scope: AutoCompactTokenLimitScope,
        pub model_provider_id: String,
        pub model_provider: ModelProviderInfo,
        pub personality: Option<Personality>,
        pub permissions: Permissions,
        pub explicit_permission_profile_mode: bool,
        pub custom_permission_profiles: Vec<CustomPermissionProfileSummary>,
        pub approvals_reviewer: ApprovalsReviewer,
        pub enforce_residency: Constrained<Option<codex_config::ResidencyRequirement>>,
        pub hide_agent_reasoning: bool,
        pub show_raw_agent_reasoning: bool,
        pub user_instructions: Option<LoadedAgentsMd>,
        pub base_instructions: Option<String>,
        pub developer_instructions: Option<String>,
        pub guardian_policy_config: Option<String>,
        pub include_permissions_instructions: bool,
        pub include_apps_instructions: bool,
        pub include_collaboration_mode_instructions: bool,
        pub include_skill_instructions: bool,
        pub include_environment_context: bool,
        pub compact_prompt: Option<String>,
        pub notify: Option<Vec<String>>,
        pub tui_notifications: TuiNotificationSettings,
        pub animations: bool,
        pub show_tooltips: bool,
        pub model_availability_nux: ModelAvailabilityNuxConfig,
        pub tui_vim_mode_default: bool,
        pub tui_raw_output_mode: bool,
        pub tui_alternate_screen: AltScreenMode,
        pub tui_status_line: Option<Vec<String>>,
        pub tui_status_line_use_colors: bool,
        pub tui_terminal_title: Option<Vec<String>>,
        pub tui_theme: Option<String>,
        pub tui_pet: Option<String>,
        pub tui_pet_anchor: TuiPetAnchor,
        pub tui_session_picker_view: SessionPickerViewMode,
        pub terminal_resize_reflow: TerminalResizeReflowConfig,
        pub tui_keymap: TuiKeymap,
        pub cwd: AbsolutePathBuf,
        pub workspace_roots: Vec<AbsolutePathBuf>,
        pub workspace_roots_explicit: bool,
        pub cli_auth_credentials_store_mode: AuthCredentialsStoreMode,
        pub mcp_servers: Constrained<HashMap<String, McpServerConfig>>,
        pub mcp_oauth_credentials_store_mode: OAuthCredentialsStoreMode,
        pub mcp_oauth_callback_port: Option<u16>,
        pub mcp_oauth_callback_url: Option<String>,
        pub model_providers: HashMap<String, ModelProviderInfo>,
        pub project_doc_max_bytes: usize,
        pub project_doc_fallback_filenames: Vec<String>,
        pub tool_output_token_limit: Option<usize>,
        pub agent_max_threads: Option<usize>,
        pub agent_job_max_runtime_seconds: Option<u64>,
        pub agent_interrupt_message_enabled: bool,
        pub agent_max_depth: i32,
        pub agent_roles: BTreeMap<String, AgentRoleConfig>,
        pub memories: MemoriesConfig,
        pub codex_home: AbsolutePathBuf,
        pub sqlite_home: PathBuf,
        pub log_dir: PathBuf,
        pub config_lock_export_dir: Option<AbsolutePathBuf>,
        pub config_lock_allow_codex_version_mismatch: bool,
        pub config_lock_save_fields_resolved_from_model_catalog: bool,
        pub config_lock_toml: Option<Arc<ConfigLockfileToml>>,
        pub history: History,
        pub ephemeral: bool,
        pub bypass_hook_trust: bool,
        pub file_opener: UriBasedFileOpener,
        pub codex_self_exe: Option<PathBuf>,
        pub codex_linux_sandbox_exe: Option<PathBuf>,
        pub main_execve_wrapper_exe: Option<PathBuf>,
        pub zsh_path: Option<PathBuf>,
        pub model_reasoning_effort: Option<ReasoningEffort>,
        pub plan_mode_reasoning_effort: Option<ReasoningEffort>,
        pub model_reasoning_summary: Option<ReasoningSummary>,
        pub model_supports_reasoning_summaries: Option<bool>,
        pub model_catalog: Option<ModelsResponse>,
        pub model_verbosity: Option<Verbosity>,
        pub chatgpt_base_url: String,
        pub apps_mcp_path_override: Option<String>,
        pub apps_mcp_product_sku: Option<String>,
        pub realtime_audio: RealtimeAudioConfig,
        pub experimental_realtime_ws_base_url: Option<String>,
        pub experimental_realtime_ws_model: Option<String>,
        pub realtime: RealtimeConfig,
        pub experimental_realtime_ws_backend_prompt: Option<String>,
        pub experimental_realtime_ws_startup_context: Option<String>,
        pub experimental_realtime_start_instructions: Option<String>,
        pub experimental_thread_config_endpoint: Option<String>,
        pub experimental_thread_store: ThreadStoreConfig,
        pub forced_chatgpt_workspace_id: Option<Vec<String>>,
        pub forced_login_method: Option<ForcedLoginMethod>,
        pub web_search_mode: Constrained<WebSearchMode>,
        pub web_search_config: Option<WebSearchConfig>,
        pub experimental_request_user_input_enabled: bool,
        pub code_mode: CodeModeConfig,
        pub use_experimental_unified_exec_tool: bool,
        pub background_terminal_max_timeout: u64,
        pub ghost_snapshot: GhostSnapshotConfig,
        pub multi_agent_v2: MultiAgentV2Config,
        pub features: ManagedFeatures,
        pub suppress_unstable_features_warning: bool,
        pub active_project: ProjectConfig,
        pub notices: Notice,
        pub check_for_update_on_startup: bool,
        pub disable_paste_burst: bool,
        pub analytics_enabled: Option<bool>,
        pub feedback_enabled: bool,
        pub tool_suggest: ToolSuggestConfig,
        pub otel: OtelConfig,
        pub toml: Option<TomlValue>,
        pub global: bool,
    }

    impl Default for Config {
        fn default() -> Self {
            let model_providers = built_in_model_providers(None);
            let model_provider = model_providers
                .get(OPENAI_PROVIDER_ID)
                .cloned()
                .unwrap_or_default();
            let cwd = AbsolutePathBuf::resolve_path_against_base("/project", "/");
            let codex_home = AbsolutePathBuf::resolve_path_against_base("/.codex", "/");
            Self {
                config_layer_stack: ConfigLayerStack::default(),
                startup_warnings: Vec::new(),
                model: None,
                service_tier: Some(SERVICE_TIER_DEFAULT_REQUEST_VALUE.to_string()),
                review_model: None,
                model_context_window: None,
                model_auto_compact_token_limit: None,
                model_auto_compact_token_limit_scope: AutoCompactTokenLimitScope::default(),
                model_provider_id: OPENAI_PROVIDER_ID.to_string(),
                model_provider,
                personality: None,
                permissions: Permissions::default(),
                explicit_permission_profile_mode: false,
                custom_permission_profiles: Vec::new(),
                approvals_reviewer: ApprovalsReviewer::default(),
                enforce_residency: Constrained::allow_any(None),
                hide_agent_reasoning: false,
                show_raw_agent_reasoning: false,
                user_instructions: None,
                base_instructions: None,
                developer_instructions: None,
                guardian_policy_config: None,
                include_permissions_instructions: true,
                include_apps_instructions: true,
                include_collaboration_mode_instructions: true,
                include_skill_instructions: true,
                include_environment_context: true,
                compact_prompt: None,
                notify: None,
                tui_notifications: TuiNotificationSettings::default(),
                animations: false,
                show_tooltips: true,
                model_availability_nux: ModelAvailabilityNuxConfig::default(),
                tui_vim_mode_default: false,
                tui_raw_output_mode: false,
                tui_alternate_screen: AltScreenMode::Auto,
                tui_status_line: None,
                tui_status_line_use_colors: true,
                tui_terminal_title: None,
                tui_theme: None,
                tui_pet: None,
                tui_pet_anchor: TuiPetAnchor::default(),
                tui_session_picker_view: SessionPickerViewMode::default(),
                terminal_resize_reflow: TerminalResizeReflowConfig::default(),
                tui_keymap: TuiKeymap::default(),
                cwd: cwd.clone(),
                workspace_roots: vec![cwd.clone()],
                workspace_roots_explicit: false,
                cli_auth_credentials_store_mode: AuthCredentialsStoreMode::default(),
                mcp_servers: Constrained::allow_any(HashMap::new()),
                mcp_oauth_credentials_store_mode: OAuthCredentialsStoreMode::default(),
                mcp_oauth_callback_port: None,
                mcp_oauth_callback_url: None,
                model_providers,
                project_doc_max_bytes: 32 * 1024,
                project_doc_fallback_filenames: Vec::new(),
                tool_output_token_limit: None,
                agent_max_threads: None,
                agent_job_max_runtime_seconds: None,
                agent_interrupt_message_enabled: true,
                agent_max_depth: 4,
                agent_roles: BTreeMap::new(),
                memories: MemoriesConfig::default(),
                codex_home: codex_home.clone(),
                sqlite_home: codex_home.to_path_buf(),
                log_dir: codex_home.to_path_buf(),
                config_lock_export_dir: None,
                config_lock_allow_codex_version_mismatch: false,
                config_lock_save_fields_resolved_from_model_catalog: false,
                config_lock_toml: None,
                history: History::default(),
                ephemeral: true,
                bypass_hook_trust: false,
                file_opener: UriBasedFileOpener::default(),
                codex_self_exe: None,
                codex_linux_sandbox_exe: None,
                main_execve_wrapper_exe: None,
                zsh_path: None,
                model_reasoning_effort: None,
                plan_mode_reasoning_effort: None,
                model_reasoning_summary: None,
                model_supports_reasoning_summaries: None,
                model_catalog: None,
                model_verbosity: None,
                chatgpt_base_url: "https://chatgpt.com".to_string(),
                apps_mcp_path_override: None,
                apps_mcp_product_sku: None,
                realtime_audio: RealtimeAudioConfig::default(),
                experimental_realtime_ws_base_url: None,
                experimental_realtime_ws_model: None,
                realtime: RealtimeConfig::default(),
                experimental_realtime_ws_backend_prompt: None,
                experimental_realtime_ws_startup_context: None,
                experimental_realtime_start_instructions: None,
                experimental_thread_config_endpoint: None,
                experimental_thread_store: ThreadStoreConfig::default(),
                forced_chatgpt_workspace_id: None,
                forced_login_method: None,
                web_search_mode: Constrained::allow_any(WebSearchMode::default()),
                web_search_config: None,
                experimental_request_user_input_enabled: false,
                code_mode: CodeModeConfig::default(),
                use_experimental_unified_exec_tool: false,
                background_terminal_max_timeout: 300_000,
                ghost_snapshot: GhostSnapshotConfig::default(),
                multi_agent_v2: MultiAgentV2Config::default(),
                features: ManagedFeatures::default(),
                suppress_unstable_features_warning: false,
                active_project: ProjectConfig::default(),
                notices: Notice::default(),
                check_for_update_on_startup: false,
                disable_paste_burst: false,
                analytics_enabled: None,
                feedback_enabled: true,
                tool_suggest: ToolSuggestConfig::default(),
                otel: OtelConfig::default(),
                toml: None,
                global: false,
            }
        }
    }

    impl Config {
        pub fn effective_workspace_roots(&self) -> Vec<AbsolutePathBuf> {
            let mut roots = if self.workspace_roots.is_empty() {
                vec![self.cwd.clone()]
            } else {
                self.workspace_roots.clone()
            };
            for root in self.permissions.profile_workspace_roots() {
                if !roots.contains(root) {
                    roots.push(root.clone());
                }
            }
            roots
        }
    }

    #[derive(Clone, Default)]
    pub struct ConfigBuilder {
        config: Config,
    }

    impl ConfigBuilder {
        pub fn new(_codex_home: PathBuf) -> Self {
            Self::default()
        }

        pub async fn build(self) -> std::io::Result<Config> {
            Ok(self.config)
        }
    }

    #[derive(Debug, Clone, Default, PartialEq)]
    pub struct ConfigOverrides;

    pub mod edit {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct LoadedAgentsMd {
        pub contents: String,
        pub path: PathBuf,
    }

    #[allow(dead_code)]
    fn _core_approvals_reviewer(_: CoreApprovalsReviewer) {}

    #[allow(dead_code)]
    fn _sandbox_mode(_: SandboxMode, _: ServiceTier) {}
}

pub mod connectors {}
pub mod otel_init {}
pub mod personality_migration {}
pub mod review_format {}
pub mod review_prompts {}
pub mod test_support {}
