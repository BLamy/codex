use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use codex_protocol::ThreadId;
use codex_protocol::dynamic_tools::DynamicToolSpec;
use codex_protocol::models::BaseInstructions;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::InitialHistory;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::RolloutItem;
pub use codex_protocol::protocol::SessionMeta;
use codex_protocol::protocol::SessionMetaLine;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::ThreadSource;
use codex_state::ThreadMetadataBuilder;

const BROWSER_STORAGE_SHIM_REQUIRED: &str =
    "browser rollout persistence requires an almostnode storage host shim";

pub const SESSIONS_SUBDIR: &str = "sessions";
pub const ARCHIVED_SESSIONS_SUBDIR: &str = "archived_sessions";
pub static INTERACTIVE_SESSION_SOURCES: LazyLock<Vec<SessionSource>> = LazyLock::new(|| {
    vec![
        SessionSource::Cli,
        SessionSource::VSCode,
        SessionSource::Custom("atlas".to_string()),
        SessionSource::Custom("chatgpt".to_string()),
    ]
});

pub trait RolloutConfigView {
    fn codex_home(&self) -> &Path;
    fn sqlite_home(&self) -> &Path;
    fn cwd(&self) -> &Path;
    fn model_provider_id(&self) -> &str;
    fn generate_memories(&self) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RolloutConfig {
    pub codex_home: PathBuf,
    pub sqlite_home: PathBuf,
    pub cwd: PathBuf,
    pub model_provider_id: String,
    pub generate_memories: bool,
}

pub type Config = RolloutConfig;

impl RolloutConfig {
    pub fn from_view(view: &impl RolloutConfigView) -> Self {
        Self {
            codex_home: view.codex_home().to_path_buf(),
            sqlite_home: view.sqlite_home().to_path_buf(),
            cwd: view.cwd().to_path_buf(),
            model_provider_id: view.model_provider_id().to_string(),
            generate_memories: view.generate_memories(),
        }
    }
}

impl RolloutConfigView for RolloutConfig {
    fn codex_home(&self) -> &Path {
        self.codex_home.as_path()
    }

    fn sqlite_home(&self) -> &Path {
        self.sqlite_home.as_path()
    }

    fn cwd(&self) -> &Path {
        self.cwd.as_path()
    }

    fn model_provider_id(&self) -> &str {
        self.model_provider_id.as_str()
    }

    fn generate_memories(&self) -> bool {
        self.generate_memories
    }
}

impl<T: RolloutConfigView + ?Sized> RolloutConfigView for &T {
    fn codex_home(&self) -> &Path {
        (*self).codex_home()
    }

    fn sqlite_home(&self) -> &Path {
        (*self).sqlite_home()
    }

    fn cwd(&self) -> &Path {
        (*self).cwd()
    }

    fn model_provider_id(&self) -> &str {
        (*self).model_provider_id()
    }

    fn generate_memories(&self) -> bool {
        (*self).generate_memories()
    }
}

impl<T: RolloutConfigView + ?Sized> RolloutConfigView for Arc<T> {
    fn codex_home(&self) -> &Path {
        self.as_ref().codex_home()
    }

    fn sqlite_home(&self) -> &Path {
        self.as_ref().sqlite_home()
    }

    fn cwd(&self) -> &Path {
        self.as_ref().cwd()
    }

    fn model_provider_id(&self) -> &str {
        self.as_ref().model_provider_id()
    }

    fn generate_memories(&self) -> bool {
        self.as_ref().generate_memories()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor;

impl serde::Serialize for Cursor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("")
    }
}

impl<'de> serde::Deserialize<'de> for Cursor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _ = String::deserialize(deserializer)?;
        Ok(Cursor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadSortKey {
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadListLayout {
    NestedByDate,
    Flat,
}

pub struct ThreadListConfig<'a> {
    pub allowed_sources: &'a [SessionSource],
    pub model_providers: Option<&'a [String]>,
    pub cwd_filters: Option<&'a [PathBuf]>,
    pub default_provider: &'a str,
    pub layout: ThreadListLayout,
}

#[derive(Debug, Default, PartialEq)]
pub struct ThreadsPage {
    pub items: Vec<ThreadItem>,
    pub next_cursor: Option<Cursor>,
    pub num_scanned_files: usize,
    pub reached_scan_cap: bool,
}

#[derive(Debug, PartialEq, Default)]
pub struct ThreadItem {
    pub path: PathBuf,
    pub thread_id: Option<ThreadId>,
    pub first_user_message: Option<String>,
    pub preview: Option<String>,
    pub cwd: Option<PathBuf>,
    pub git_branch: Option<String>,
    pub git_sha: Option<String>,
    pub git_origin_url: Option<String>,
    pub source: Option<SessionSource>,
    pub parent_thread_id: Option<ThreadId>,
    pub agent_nickname: Option<String>,
    pub agent_role: Option<String>,
    pub model_provider: Option<String>,
    pub cli_version: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

pub type RolloutSearchMatches = HashMap<PathBuf, Option<String>>;

#[derive(Clone)]
pub struct RolloutRecorder {
    rollout_path: PathBuf,
}

#[derive(Clone)]
pub enum RolloutRecorderParams {
    Create {
        conversation_id: ThreadId,
        forked_from_id: Option<ThreadId>,
        parent_thread_id: Option<ThreadId>,
        source: SessionSource,
        thread_source: Option<ThreadSource>,
        base_instructions: BaseInstructions,
        dynamic_tools: Vec<DynamicToolSpec>,
        multi_agent_version: Option<MultiAgentVersion>,
    },
    Resume {
        path: PathBuf,
    },
}

impl RolloutRecorderParams {
    pub fn new(
        conversation_id: ThreadId,
        forked_from_id: Option<ThreadId>,
        parent_thread_id: Option<ThreadId>,
        source: SessionSource,
        thread_source: Option<ThreadSource>,
        base_instructions: BaseInstructions,
        dynamic_tools: Vec<DynamicToolSpec>,
    ) -> Self {
        Self::Create {
            conversation_id,
            forked_from_id,
            parent_thread_id,
            source,
            thread_source,
            base_instructions,
            dynamic_tools,
            multi_agent_version: None,
        }
    }

    pub fn with_multi_agent_version(
        mut self,
        multi_agent_version: Option<MultiAgentVersion>,
    ) -> Self {
        if let Self::Create {
            multi_agent_version: version,
            ..
        } = &mut self
        {
            *version = multi_agent_version;
        }
        self
    }

    pub fn resume(path: PathBuf) -> Self {
        Self::Resume { path }
    }
}

impl RolloutRecorder {
    #[allow(clippy::too_many_arguments)]
    pub async fn list_threads(
        state_db_ctx: Option<StateDbHandle>,
        config: &impl RolloutConfigView,
        page_size: usize,
        cursor: Option<&Cursor>,
        sort_key: ThreadSortKey,
        sort_direction: SortDirection,
        allowed_sources: &[SessionSource],
        model_providers: Option<&[String]>,
        cwd_filters: Option<&[PathBuf]>,
        default_provider: &str,
        search_term: Option<&str>,
    ) -> io::Result<ThreadsPage> {
        let _ = (
            state_db_ctx,
            config.codex_home(),
            page_size,
            cursor,
            sort_key,
            sort_direction,
            allowed_sources,
            model_providers,
            cwd_filters,
            default_provider,
            search_term,
        );
        Ok(ThreadsPage::default())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_threads_from_state_db(
        state_db_ctx: Option<StateDbHandle>,
        config: &impl RolloutConfigView,
        page_size: usize,
        cursor: Option<&Cursor>,
        sort_key: ThreadSortKey,
        sort_direction: SortDirection,
        allowed_sources: &[SessionSource],
        model_providers: Option<&[String]>,
        cwd_filters: Option<&[PathBuf]>,
        default_provider: &str,
        search_term: Option<&str>,
    ) -> io::Result<ThreadsPage> {
        Self::list_threads(
            state_db_ctx,
            config,
            page_size,
            cursor,
            sort_key,
            sort_direction,
            allowed_sources,
            model_providers,
            cwd_filters,
            default_provider,
            search_term,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_archived_threads(
        state_db_ctx: Option<StateDbHandle>,
        config: &impl RolloutConfigView,
        page_size: usize,
        cursor: Option<&Cursor>,
        sort_key: ThreadSortKey,
        sort_direction: SortDirection,
        allowed_sources: &[SessionSource],
        model_providers: Option<&[String]>,
        cwd_filters: Option<&[PathBuf]>,
        default_provider: &str,
        search_term: Option<&str>,
    ) -> io::Result<ThreadsPage> {
        Self::list_threads(
            state_db_ctx,
            config,
            page_size,
            cursor,
            sort_key,
            sort_direction,
            allowed_sources,
            model_providers,
            cwd_filters,
            default_provider,
            search_term,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_archived_threads_from_state_db(
        state_db_ctx: Option<StateDbHandle>,
        config: &impl RolloutConfigView,
        page_size: usize,
        cursor: Option<&Cursor>,
        sort_key: ThreadSortKey,
        sort_direction: SortDirection,
        allowed_sources: &[SessionSource],
        model_providers: Option<&[String]>,
        cwd_filters: Option<&[PathBuf]>,
        default_provider: &str,
        search_term: Option<&str>,
    ) -> io::Result<ThreadsPage> {
        Self::list_threads(
            state_db_ctx,
            config,
            page_size,
            cursor,
            sort_key,
            sort_direction,
            allowed_sources,
            model_providers,
            cwd_filters,
            default_provider,
            search_term,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn find_latest_thread_path(
        state_db_ctx: Option<StateDbHandle>,
        config: &impl RolloutConfigView,
        page_size: usize,
        cursor: Option<&Cursor>,
        sort_key: ThreadSortKey,
        allowed_sources: &[SessionSource],
        model_providers: Option<&[String]>,
        default_provider: &str,
        filter_cwd: Option<&Path>,
    ) -> io::Result<Option<PathBuf>> {
        let _ = (
            state_db_ctx,
            config.codex_home(),
            page_size,
            cursor,
            sort_key,
            allowed_sources,
            model_providers,
            default_provider,
            filter_cwd,
        );
        Ok(None)
    }

    pub async fn new(
        config: &impl RolloutConfigView,
        params: RolloutRecorderParams,
    ) -> io::Result<Self> {
        let rollout_path = match params {
            RolloutRecorderParams::Create {
                conversation_id, ..
            } => config
                .codex_home()
                .join(SESSIONS_SUBDIR)
                .join(format!("browser-rollout-{conversation_id}.jsonl")),
            RolloutRecorderParams::Resume { path } => path,
        };
        Ok(Self { rollout_path })
    }

    pub fn rollout_path(&self) -> &Path {
        self.rollout_path.as_path()
    }

    pub async fn record_canonical_items(&self, items: &[RolloutItem]) -> io::Result<()> {
        let _ = items;
        Ok(())
    }

    pub async fn persist(&self) -> io::Result<()> {
        Ok(())
    }

    pub async fn flush(&self) -> io::Result<()> {
        Ok(())
    }

    pub async fn load_rollout_items(
        path: &Path,
    ) -> io::Result<(Vec<RolloutItem>, Option<ThreadId>, usize)> {
        Err(io::Error::other(format!(
            "{BROWSER_STORAGE_SHIM_REQUIRED}: cannot read {}",
            path.display()
        )))
    }

    pub async fn get_rollout_history(path: &Path) -> io::Result<InitialHistory> {
        Err(io::Error::other(format!(
            "{BROWSER_STORAGE_SHIM_REQUIRED}: cannot read {}",
            path.display()
        )))
    }

    pub async fn shutdown(&self) -> io::Result<()> {
        Ok(())
    }
}

pub async fn append_rollout_item_to_path(
    _rollout_path: &Path,
    _item: &RolloutItem,
) -> io::Result<()> {
    Ok(())
}

pub fn spawn_rollout_compression_worker(codex_home: PathBuf) {
    let _ = codex_home;
}

pub struct RolloutLineReader;

impl RolloutLineReader {
    pub async fn next_line(&mut self) -> io::Result<Option<String>> {
        Ok(None)
    }
}

pub async fn open_rollout_line_reader(_path: &Path) -> io::Result<RolloutLineReader> {
    Ok(RolloutLineReader)
}

pub async fn existing_rollout_path(path: &Path) -> Option<PathBuf> {
    Some(path.to_path_buf())
}

pub fn plain_rollout_path(path: &Path) -> PathBuf {
    path.to_path_buf()
}

pub async fn get_threads(
    codex_home: &Path,
    page_size: usize,
    cursor: Option<&Cursor>,
    sort_key: ThreadSortKey,
    allowed_sources: &[SessionSource],
    model_providers: Option<&[String]>,
    cwd_filters: Option<&[PathBuf]>,
    default_provider: &str,
) -> io::Result<ThreadsPage> {
    let _ = (
        codex_home,
        page_size,
        cursor,
        sort_key,
        allowed_sources,
        model_providers,
        cwd_filters,
        default_provider,
    );
    Ok(ThreadsPage::default())
}

pub async fn get_threads_in_root(
    root: PathBuf,
    page_size: usize,
    cursor: Option<&Cursor>,
    sort_key: ThreadSortKey,
    config: ThreadListConfig<'_>,
) -> io::Result<ThreadsPage> {
    let _ = (root, page_size, cursor, sort_key, config);
    Ok(ThreadsPage::default())
}

pub fn parse_cursor(_token: &str) -> Option<Cursor> {
    None
}

pub async fn read_thread_item_from_rollout(path: PathBuf) -> Option<ThreadItem> {
    let _ = path;
    None
}

pub async fn read_head_for_summary(_path: &Path) -> io::Result<Vec<serde_json::Value>> {
    Ok(Vec::new())
}

pub async fn read_session_meta_line(path: &Path) -> io::Result<SessionMetaLine> {
    Err(io::Error::other(format!(
        "{BROWSER_STORAGE_SHIM_REQUIRED}: cannot read {}",
        path.display()
    )))
}

pub async fn find_thread_path_by_id_str(
    codex_home: &Path,
    id_str: &str,
    state_db_ctx: Option<&codex_state::StateRuntime>,
) -> io::Result<Option<PathBuf>> {
    let _ = (codex_home, id_str, state_db_ctx);
    Ok(None)
}

pub async fn find_archived_thread_path_by_id_str(
    codex_home: &Path,
    id_str: &str,
    state_db_ctx: Option<&codex_state::StateRuntime>,
) -> io::Result<Option<PathBuf>> {
    let _ = (codex_home, id_str, state_db_ctx);
    Ok(None)
}

#[deprecated(note = "use find_thread_path_by_id_str")]
pub async fn find_conversation_path_by_id_str(
    codex_home: &Path,
    id_str: &str,
    state_db_ctx: Option<&codex_state::StateRuntime>,
) -> io::Result<Option<PathBuf>> {
    find_thread_path_by_id_str(codex_home, id_str, state_db_ctx).await
}

pub async fn append_thread_name(
    codex_home: &Path,
    thread_id: ThreadId,
    name: &str,
) -> io::Result<()> {
    let _ = (codex_home, thread_id, name);
    Ok(())
}

pub async fn find_thread_name_by_id(
    codex_home: &Path,
    id: &ThreadId,
) -> io::Result<Option<String>> {
    let _ = (codex_home, id);
    Ok(None)
}

pub async fn find_thread_names_by_ids(
    codex_home: &Path,
    ids: &HashSet<ThreadId>,
) -> io::Result<HashMap<ThreadId, String>> {
    let _ = (codex_home, ids);
    Ok(HashMap::new())
}

pub async fn find_thread_meta_by_name_str(
    codex_home: &Path,
    name: &str,
    state_db_ctx: Option<&codex_state::StateRuntime>,
) -> io::Result<Option<(PathBuf, SessionMetaLine)>> {
    let _ = (codex_home, name, state_db_ctx);
    Ok(None)
}

pub fn rollout_date_parts(file_name: &OsStr) -> Option<(String, String, String)> {
    let name = file_name.to_string_lossy();
    let date = name.strip_prefix("rollout-")?.get(..10)?;
    Some((
        date.get(..4)?.to_string(),
        date.get(5..7)?.to_string(),
        date.get(8..10)?.to_string(),
    ))
}

pub fn builder_from_items(
    _items: &[RolloutItem],
    _rollout_path: &Path,
) -> Option<ThreadMetadataBuilder> {
    None
}

pub fn is_persisted_rollout_item(item: &RolloutItem) -> bool {
    match item {
        RolloutItem::ResponseItem(item) => should_persist_response_item(item),
        RolloutItem::EventMsg(ev) => should_persist_event_msg(ev),
        RolloutItem::Compacted(_) | RolloutItem::TurnContext(_) | RolloutItem::SessionMeta(_) => {
            true
        }
    }
}

pub fn persisted_rollout_items(items: &[RolloutItem]) -> Vec<RolloutItem> {
    items
        .iter()
        .filter(|item| is_persisted_rollout_item(item))
        .cloned()
        .collect()
}

pub fn should_persist_response_item(item: &ResponseItem) -> bool {
    !matches!(item, ResponseItem::CompactionTrigger | ResponseItem::Other)
}

pub fn should_persist_response_item_for_memories(item: &ResponseItem) -> bool {
    match item {
        ResponseItem::Message { role, .. } => role != "developer",
        ResponseItem::LocalShellCall { .. }
        | ResponseItem::FunctionCall { .. }
        | ResponseItem::ToolSearchCall { .. }
        | ResponseItem::FunctionCallOutput { .. }
        | ResponseItem::ToolSearchOutput { .. }
        | ResponseItem::CustomToolCall { .. }
        | ResponseItem::CustomToolCallOutput { .. }
        | ResponseItem::WebSearchCall { .. } => true,
        ResponseItem::Reasoning { .. }
        | ResponseItem::ImageGenerationCall { .. }
        | ResponseItem::Compaction { .. }
        | ResponseItem::CompactionTrigger
        | ResponseItem::ContextCompaction { .. }
        | ResponseItem::Other => false,
    }
}

pub fn should_persist_event_msg(ev: &EventMsg) -> bool {
    matches!(
        ev,
        EventMsg::UserMessage(_)
            | EventMsg::AgentMessage(_)
            | EventMsg::AgentReasoning(_)
            | EventMsg::AgentReasoningRawContent(_)
            | EventMsg::PatchApplyEnd(_)
            | EventMsg::TokenCount(_)
            | EventMsg::ThreadGoalUpdated(_)
            | EventMsg::ContextCompacted(_)
            | EventMsg::EnteredReviewMode(_)
            | EventMsg::ExitedReviewMode(_)
            | EventMsg::McpToolCallEnd(_)
            | EventMsg::ThreadRolledBack(_)
            | EventMsg::TurnAborted(_)
            | EventMsg::TurnStarted(_)
            | EventMsg::TurnComplete(_)
            | EventMsg::WebSearchEnd(_)
            | EventMsg::ImageGenerationEnd(_)
    )
}

pub async fn search_rollout_paths(
    rg_command: &Path,
    codex_home: &Path,
    archived: bool,
    search_term: &str,
) -> io::Result<HashSet<PathBuf>> {
    let _ = (rg_command, codex_home, archived, search_term);
    Ok(HashSet::new())
}

pub async fn search_rollout_matches(
    rg_command: &Path,
    codex_home: &Path,
    archived: bool,
    search_term: &str,
) -> io::Result<RolloutSearchMatches> {
    let _ = (rg_command, codex_home, archived, search_term);
    Ok(HashMap::new())
}

pub async fn first_rollout_content_match_snippet(
    path: &Path,
    search_term: &str,
) -> io::Result<Option<String>> {
    let _ = (path, search_term);
    Ok(None)
}

pub type StateDbHandle = Arc<codex_state::StateRuntime>;

pub mod state_db {
    use super::*;

    pub type StateDbHandle = super::StateDbHandle;
    pub use codex_state::LogEntry;

    pub async fn init(_config: &impl RolloutConfigView) -> Option<StateDbHandle> {
        None
    }

    pub async fn try_init(_config: &impl RolloutConfigView) -> anyhow::Result<StateDbHandle> {
        anyhow::bail!(BROWSER_STORAGE_SHIM_REQUIRED)
    }

    pub async fn get_state_db(_config: &impl RolloutConfigView) -> Option<StateDbHandle> {
        None
    }

    pub fn sqlite_telemetry_recorder(
        _metrics: codex_otel::MetricsClient,
        _originator: &str,
    ) -> codex_state::DbTelemetryHandle {
        Arc::new(NoopDbTelemetry)
    }

    pub fn normalize_cwd_for_state_db(cwd: &Path) -> PathBuf {
        cwd.to_path_buf()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_thread_ids_db(
        context: Option<&codex_state::StateRuntime>,
        codex_home: &Path,
        page_size: usize,
        cursor: Option<&Cursor>,
        sort_key: ThreadSortKey,
        allowed_sources: &[SessionSource],
        model_providers: Option<&[String]>,
        archived_only: bool,
        stage: &str,
    ) -> Option<Vec<ThreadId>> {
        let _ = (
            context,
            codex_home,
            page_size,
            cursor,
            sort_key,
            allowed_sources,
            model_providers,
            archived_only,
            stage,
        );
        None
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_threads_db(
        context: Option<&codex_state::StateRuntime>,
        codex_home: &Path,
        page_size: usize,
        cursor: Option<&Cursor>,
        sort_key: ThreadSortKey,
        sort_direction: SortDirection,
        allowed_sources: &[SessionSource],
        model_providers: Option<&[String]>,
        cwd_filters: Option<&[PathBuf]>,
        archived: bool,
        search_term: Option<&str>,
    ) -> Option<codex_state::ThreadsPage> {
        let _ = (
            context,
            codex_home,
            page_size,
            cursor,
            sort_key,
            sort_direction,
            allowed_sources,
            model_providers,
            cwd_filters,
            archived,
            search_term,
        );
        None
    }

    pub async fn find_rollout_path_by_id(
        context: Option<&codex_state::StateRuntime>,
        thread_id: ThreadId,
        archived_only: Option<bool>,
        stage: &str,
    ) -> Option<PathBuf> {
        let _ = (context, thread_id, archived_only, stage);
        None
    }

    pub async fn mark_thread_memory_mode_polluted(
        context: Option<&codex_state::StateRuntime>,
        thread_id: ThreadId,
        stage: &str,
    ) {
        let _ = (context, thread_id, stage);
    }

    pub async fn reconcile_rollout(
        context: Option<&codex_state::StateRuntime>,
        rollout_path: &Path,
        default_provider: &str,
        builder: Option<&ThreadMetadataBuilder>,
        items: &[RolloutItem],
        archived_only: Option<bool>,
        new_thread_memory_mode: Option<&str>,
    ) {
        let _ = (
            context,
            rollout_path,
            default_provider,
            builder,
            items,
            archived_only,
            new_thread_memory_mode,
        );
    }

    pub async fn read_repair_rollout_path(
        context: Option<&codex_state::StateRuntime>,
        thread_id: Option<ThreadId>,
        archived_only: Option<bool>,
        rollout_path: &Path,
    ) {
        let _ = (context, thread_id, archived_only, rollout_path);
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn apply_rollout_items(
        context: Option<&codex_state::StateRuntime>,
        rollout_path: &Path,
        default_provider: &str,
        builder: Option<&ThreadMetadataBuilder>,
        items: &[RolloutItem],
        stage: &str,
        new_thread_memory_mode: Option<&str>,
        updated_at_override: Option<DateTime<Utc>>,
    ) {
        let _ = (
            context,
            rollout_path,
            default_provider,
            builder,
            items,
            stage,
            new_thread_memory_mode,
            updated_at_override,
        );
    }

    pub async fn touch_thread_updated_at(
        context: Option<&codex_state::StateRuntime>,
        thread_id: Option<ThreadId>,
        updated_at: DateTime<Utc>,
        stage: &str,
    ) -> bool {
        let _ = (context, thread_id, updated_at, stage);
        false
    }

    struct NoopDbTelemetry;

    impl codex_state::DbTelemetry for NoopDbTelemetry {
        fn counter(&self, _name: &str, _inc: i64, _tags: &[(&str, &str)]) {}

        fn record_duration(&self, _name: &str, _duration: Duration, _tags: &[(&str, &str)]) {}
    }
}

pub use state_db::sqlite_telemetry_recorder;
