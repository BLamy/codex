use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    UnderDevelopment,
    Experimental {
        name: &'static str,
        menu_description: &'static str,
        announcement: &'static str,
    },
    Stable,
    Deprecated,
    Removed,
}

impl Stage {
    pub fn experimental_menu_name(self) -> Option<&'static str> {
        match self {
            Stage::Experimental { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn experimental_menu_description(self) -> Option<&'static str> {
        match self {
            Stage::Experimental {
                menu_description, ..
            } => Some(menu_description),
            _ => None,
        }
    }

    pub fn experimental_announcement(self) -> Option<&'static str> {
        match self {
            Stage::Experimental {
                announcement: "", ..
            } => None,
            Stage::Experimental { announcement, .. } => Some(announcement),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    ShellTool,
    CodexHooks,
    CodeMode,
    CodeModeOnly,
    UnifiedExec,
    ShellZshFork,
    UnifiedExecZshFork,
    TerminalResizeReflow,
    ApplyPatchStreamingEvents,
    ExecPermissionApprovals,
    RequestPermissionsTool,
    WebSearchRequest,
    WebSearchCached,
    StandaloneWebSearch,
    UseLegacyLandlock,
    ShellSnapshot,
    RuntimeMetrics,
    MemoryTool,
    LocalThreadStoreCompression,
    Chronicle,
    ChildAgentsMd,
    EnableRequestCompression,
    NetworkProxy,
    Collab,
    MultiAgentV2,
    SpawnCsv,
    Apps,
    EnableMcpApps,
    AppsMcpPathOverride,
    ToolSearch,
    ToolSearchAlwaysDeferMcpTools,
    NonPrefixedMcpToolNames,
    ToolSuggest,
    Plugins,
    PluginHooks,
    InAppBrowser,
    BrowserUse,
    BrowserUseExternal,
    ComputerUse,
    RemotePlugin,
    PluginSharing,
    ExternalMigration,
    ImageGeneration,
    ImageGenExt,
    SkillMcpDependencyInstall,
    SkillEnvVarDependencyPrompt,
    MentionsV2,
    DefaultModeRequestUserInput,
    GuardianApproval,
    Goals,
    ToolCallMcpElicitation,
    AuthElicitation,
    Personality,
    Artifact,
    FastMode,
    RealtimeConversation,
    PreventIdleSleep,
    RemoteCompactionV2,
    WorkspaceDependencies,
    GhostCommit,
    JsRepl,
    JsReplToolsOnly,
    SearchTool,
    UseLinuxSandboxBwrap,
    RequestRule,
    WindowsSandbox,
    WindowsSandboxElevated,
    RemoteModels,
    CodexGitCommit,
    Sqlite,
    ApplyPatchFreeform,
    UnavailableDummyTools,
    Steer,
    CollaborationModes,
    RemoteControl,
    ImageDetailOriginal,
    TuiAppServer,
    WorkspaceOwnerUsageNudge,
    ResponsesWebsockets,
    ResponsesWebsocketsV2,
}

impl Feature {
    pub fn key(self) -> &'static str {
        self.info().key
    }

    pub fn stage(self) -> Stage {
        self.info().stage
    }

    pub fn default_enabled(self) -> bool {
        self.info().default_enabled
    }

    fn info(self) -> &'static FeatureSpec {
        FEATURES
            .iter()
            .find(|spec| spec.id == self)
            .unwrap_or(&FALLBACK_FEATURE_SPEC)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LegacyFeatureUsage {
    pub alias: String,
    pub feature: Feature,
    pub summary: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Features {
    enabled: BTreeSet<Feature>,
    legacy_usages: BTreeSet<LegacyFeatureUsage>,
}

impl Default for Features {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl Features {
    pub fn with_defaults() -> Self {
        let enabled = FEATURES
            .iter()
            .filter(|spec| spec.default_enabled)
            .map(|spec| spec.id)
            .collect::<BTreeSet<_>>();
        Self {
            enabled,
            legacy_usages: BTreeSet::new(),
        }
    }

    pub fn enabled(&self, feature: Feature) -> bool {
        self.enabled.contains(&feature)
    }

    pub fn apps_enabled_for_auth(&self, has_chatgpt_auth: bool) -> bool {
        self.enabled(Feature::Apps) && has_chatgpt_auth
    }

    pub fn enable(&mut self, feature: Feature) -> &mut Self {
        self.enabled.insert(feature);
        self
    }

    pub fn disable(&mut self, feature: Feature) -> &mut Self {
        self.enabled.remove(&feature);
        self
    }

    pub fn set_enabled(&mut self, feature: Feature, enabled: bool) -> &mut Self {
        if enabled {
            self.enable(feature)
        } else {
            self.disable(feature)
        }
    }

    pub fn enabled_features(&self) -> Vec<Feature> {
        self.enabled.iter().copied().collect()
    }

    pub fn legacy_feature_usages(&self) -> impl Iterator<Item = &LegacyFeatureUsage> + '_ {
        self.legacy_usages.iter()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FeaturesToml {
    #[serde(flatten)]
    entries: BTreeMap<String, bool>,
}

impl FeaturesToml {
    pub fn entries(&self) -> BTreeMap<String, bool> {
        self.entries.clone()
    }
}

impl From<BTreeMap<String, bool>> for FeaturesToml {
    fn from(entries: BTreeMap<String, bool>) -> Self {
        Self { entries }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FeatureSpec {
    pub id: Feature,
    pub key: &'static str,
    pub stage: Stage,
    pub default_enabled: bool,
}

static FALLBACK_FEATURE_SPEC: FeatureSpec = FeatureSpec {
    id: Feature::ShellTool,
    key: "unknown",
    stage: Stage::Removed,
    default_enabled: false,
};

pub const FEATURES: &[FeatureSpec] = &[
    FeatureSpec {
        id: Feature::ShellTool,
        key: "shell_tool",
        stage: Stage::Stable,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::UnifiedExec,
        key: "unified_exec",
        stage: Stage::Stable,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::TerminalResizeReflow,
        key: "terminal_resize_reflow",
        stage: Stage::Stable,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::RuntimeMetrics,
        key: "runtime_metrics",
        stage: Stage::Experimental {
            name: "Runtime metrics",
            menu_description: "Show runtime metrics in responses when available.",
            announcement: "",
        },
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::MemoryTool,
        key: "memory_tool",
        stage: Stage::Experimental {
            name: "Memory tool",
            menu_description: "Allow Codex to use project memory tools.",
            announcement: "",
        },
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::Collab,
        key: "collab",
        stage: Stage::Stable,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::Apps,
        key: "apps",
        stage: Stage::Stable,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::Plugins,
        key: "plugins",
        stage: Stage::Experimental {
            name: "Plugins",
            menu_description: "Enable plugin discovery and plugin mentions.",
            announcement: "",
        },
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::MentionsV2,
        key: "mentions_v2",
        stage: Stage::Experimental {
            name: "Mentions v2",
            menu_description: "Use the newer mention popup for tools and plugins.",
            announcement: "",
        },
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::GuardianApproval,
        key: "guardian_approval",
        stage: Stage::Stable,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::Goals,
        key: "goals",
        stage: Stage::Experimental {
            name: "Goals",
            menu_description: "Enable persisted thread goals.",
            announcement: "",
        },
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::Personality,
        key: "personality",
        stage: Stage::Experimental {
            name: "Personality",
            menu_description: "Enable personality selection in the TUI.",
            announcement: "",
        },
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::FastMode,
        key: "fast_mode",
        stage: Stage::Stable,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::RealtimeConversation,
        key: "realtime_conversation",
        stage: Stage::Experimental {
            name: "Realtime conversation",
            menu_description: "Enable realtime voice controls.",
            announcement: "",
        },
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::PreventIdleSleep,
        key: "prevent_idle_sleep",
        stage: Stage::Stable,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::WindowsSandbox,
        key: "windows_sandbox",
        stage: Stage::Stable,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::WindowsSandboxElevated,
        key: "windows_sandbox_elevated",
        stage: Stage::Stable,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::CollaborationModes,
        key: "collaboration_modes",
        stage: Stage::Stable,
        default_enabled: true,
    },
];
