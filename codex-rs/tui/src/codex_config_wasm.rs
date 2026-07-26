use codex_app_server_protocol::ConfigLayerSource;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

pub const CONFIG_TOML_FILE: &str = "config.toml";

#[path = "../../config/src/tui_keymap.rs"]
pub mod upstream_tui_keymap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintError {
    message: String,
}

impl ConstraintError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ConstraintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ConstraintError {}

pub type ConstraintResult<T = ()> = Result<T, ConstraintError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constrained<T> {
    value: T,
}

impl<T> Constrained<T> {
    pub fn allow_any(value: T) -> Self {
        Self { value }
    }

    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) -> ConstraintResult<()> {
        self.value = value;
        Ok(())
    }
}

impl<T: Clone> Constrained<T> {
    pub fn value(&self) -> T {
        self.value.clone()
    }
}

impl<T: Clone + PartialEq + fmt::Debug + 'static> Constrained<T> {
    pub fn allow_only(value: T) -> Self {
        Self { value }
    }

    pub fn can_set(&self, _value: &T) -> ConstraintResult<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResidencyRequirement {
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigLayerStackOrdering {
    LowestPrecedenceFirst,
    HighestPrecedenceFirst,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigLayerEntry {
    pub name: ConfigLayerSource,
    pub config: toml::Value,
    pub disabled_reason: Option<String>,
}

impl ConfigLayerEntry {
    pub fn is_disabled(&self) -> bool {
        self.disabled_reason.is_some()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConfigLayerStack {
    layers: Vec<ConfigLayerEntry>,
    user_config: Option<toml::Value>,
}

impl ConfigLayerStack {
    pub fn new(layers: Vec<ConfigLayerEntry>) -> Self {
        Self {
            layers,
            user_config: None,
        }
    }

    pub fn get_layers(
        &self,
        ordering: ConfigLayerStackOrdering,
        include_disabled: bool,
    ) -> Vec<ConfigLayerEntry> {
        let mut layers = self
            .layers
            .iter()
            .filter(|layer| include_disabled || !layer.is_disabled())
            .cloned()
            .collect::<Vec<_>>();
        if matches!(ordering, ConfigLayerStackOrdering::HighestPrecedenceFirst) {
            layers.reverse();
        }
        layers
    }

    pub fn get_active_user_layer(&self) -> Option<&ConfigLayerEntry> {
        self.layers.iter().rev().find(|layer| {
            !layer.is_disabled() && matches!(layer.name, ConfigLayerSource::User { .. })
        })
    }

    pub fn effective_user_config(&self) -> Option<&toml::Value> {
        self.user_config.as_ref()
    }

    pub fn with_user_config(mut self, config: toml::Value) -> Self {
        self.user_config = Some(config);
        self
    }
}

pub fn format_config_layer_source(source: &ConfigLayerSource, _config_file: &str) -> String {
    match source {
        ConfigLayerSource::Mdm { domain, key } => format!("mdm: {domain}/{key}"),
        ConfigLayerSource::System { file } => format!("system: {}", file.display()),
        ConfigLayerSource::EnterpriseManaged { name, .. } => {
            format!("enterprise managed: {name}")
        }
        ConfigLayerSource::User { file, profile } => match profile {
            Some(profile) => format!("user profile {profile}: {}", file.display()),
            None => format!("user: {}", file.display()),
        },
        ConfigLayerSource::Project { dot_codex_folder } => {
            format!("project: {}", dot_codex_folder.display())
        }
        ConfigLayerSource::SessionFlags => "session flags".to_string(),
        ConfigLayerSource::LegacyManagedConfigTomlFromFile { file } => {
            format!("legacy managed config: {}", file.display())
        }
        ConfigLayerSource::LegacyManagedConfigTomlFromMdm => "legacy managed config mdm".to_string(),
    }
}

pub mod config_toml {
    use super::types::WindowsSandboxModeToml;
    use serde::Deserialize;
    use serde::Serialize;
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ConfigLockfileToml;

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ProjectConfig {
        pub trust_level: Option<String>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct RealtimeAudioConfig {
        pub microphone: Option<String>,
        pub speaker: Option<String>,
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum RealtimeWsMode {
        #[default]
        Conversational,
        Transcription,
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum RealtimeTransport {
        #[default]
        WebRtc,
        Websocket,
    }

    #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct RealtimeConfig {
        pub version: codex_protocol::protocol::RealtimeConversationVersion,
        #[serde(rename = "type")]
        pub session_type: RealtimeWsMode,
        #[serde(default)]
        pub transport: RealtimeTransport,
        pub voice: Option<codex_protocol::protocol::RealtimeVoice>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct ConfigToml {
        pub windows: Option<WindowsToml>,
        pub marketplaces: Option<BTreeMap<String, toml::Value>>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct WindowsToml {
        pub sandbox: Option<WindowsSandboxModeToml>,
        pub sandbox_private_desktop: Option<bool>,
    }
}

pub mod types {
    pub use codex_protocol::config_types::ApprovalsReviewer;
    pub use crate::codex_config_wasm::upstream_tui_keymap::KeybindingSpec;
    pub use crate::codex_config_wasm::upstream_tui_keymap::KeybindingsSpec;
    pub use crate::codex_config_wasm::upstream_tui_keymap::MAX_FUNCTION_KEY;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiApprovalKeymap;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiChatKeymap;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiComposerKeymap;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiEditorKeymap;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiGlobalKeymap;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiKeymap;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiListKeymap;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiPagerKeymap;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiVimNormalKeymap;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiVimOperatorKeymap;
    pub use crate::codex_config_wasm::upstream_tui_keymap::TuiVimTextObjectKeymap;
    use serde::Deserialize;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::fmt;

    /// Working directory to use when resuming or forking a session.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum ResumeCwdMode {
        Current,
        Session,
    }

    impl ResumeCwdMode {
        pub const fn as_str(self) -> &'static str {
            match self {
                Self::Current => "current",
                Self::Session => "session",
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum AuthCredentialsStoreMode {
        #[default]
        File,
        Keyring,
        Auto,
        Ephemeral,
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum OAuthCredentialsStoreMode {
        #[default]
        Auto,
        File,
        Keyring,
    }

    #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct History {
        pub max_bytes: Option<usize>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct McpServerConfig {
        pub command: Option<String>,
        pub args: Option<Vec<String>>,
        pub env: Option<HashMap<String, String>>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub enum McpServerTransportConfig {
        #[default]
        Stdio,
        Sse,
        StreamableHttp,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct MemoriesConfig {
        pub use_memories: bool,
        pub generate_memories: bool,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ModelAvailabilityNuxConfig {
        pub shown_count: HashMap<String, u32>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ModelMigrationNotices {
        pub shown_count: HashMap<String, u32>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ExternalConfigMigrationPrompts {
        pub home: Option<bool>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Notice {
        pub hide_full_access_warning: Option<bool>,
        pub hide_world_writable_warning: Option<bool>,
        pub hide_rate_limit_model_nudge: Option<bool>,
        pub hide_gpt5_1_migration_prompt: Option<bool>,
        pub fast_default_opt_out: Option<bool>,
        pub model_migrations: ModelMigrationNotices,
        pub external_config_migration_prompts: ExternalConfigMigrationPrompts,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum WindowsSandboxModeToml {
        Elevated,
        Unelevated,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum UriBasedFileOpener {
        #[serde(rename = "vscode")]
        VsCode,
        #[serde(rename = "vscode-insiders")]
        VsCodeInsiders,
        #[serde(rename = "windsurf")]
        Windsurf,
        #[serde(rename = "cursor")]
        Cursor,
        #[serde(rename = "none")]
        None,
    }

    impl Default for UriBasedFileOpener {
        fn default() -> Self {
            Self::None
        }
    }

    impl UriBasedFileOpener {
        pub fn get_scheme(&self) -> Option<&str> {
            match self {
                Self::VsCode => Some("vscode"),
                Self::VsCodeInsiders => Some("vscode-insiders"),
                Self::Windsurf => Some("windsurf"),
                Self::Cursor => Some("cursor"),
                Self::None => None,
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum SessionPickerViewMode {
        Comfortable,
        #[default]
        Dense,
    }

    impl SessionPickerViewMode {
        pub const fn as_str(self) -> &'static str {
            match self {
                Self::Comfortable => "comfortable",
                Self::Dense => "dense",
            }
        }
    }

    impl fmt::Display for SessionPickerViewMode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.as_str())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Notifications {
        Enabled(bool),
        Custom(Vec<String>),
    }

    impl Default for Notifications {
        fn default() -> Self {
            Self::Enabled(true)
        }
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum NotificationMethod {
        #[default]
        Auto,
        Osc9,
        Bel,
    }

    impl fmt::Display for NotificationMethod {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                NotificationMethod::Auto => write!(f, "auto"),
                NotificationMethod::Osc9 => write!(f, "osc9"),
                NotificationMethod::Bel => write!(f, "bel"),
            }
        }
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum NotificationCondition {
        #[default]
        Unfocused,
        Always,
    }

    impl fmt::Display for NotificationCondition {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                NotificationCondition::Unfocused => write!(f, "unfocused"),
                NotificationCondition::Always => write!(f, "always"),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TuiNotificationSettings {
        pub notifications: Notifications,
        pub method: NotificationMethod,
        pub condition: NotificationCondition,
    }

    impl Default for TuiNotificationSettings {
        fn default() -> Self {
            Self {
                notifications: Notifications::default(),
                method: NotificationMethod::default(),
                condition: NotificationCondition::default(),
            }
        }
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum TuiPetAnchor {
        #[default]
        Composer,
        ScreenBottom,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ToolSuggestConfig {
        pub disabled: bool,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct OtelConfig;

    pub const DEFAULT_TERMINAL_RESIZE_REFLOW_FALLBACK_MAX_ROWS: usize = 1_000;
}
