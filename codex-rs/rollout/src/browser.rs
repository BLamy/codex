//! Browser storage boundary for rollout persistence.
//!
//! Native Codex persists JSONL rollouts and maintains a SQLite index. Neither
//! storage backend exists on `wasm32-unknown-unknown`, so the browser build
//! keeps the current process' rollouts in memory. Filesystem-backed hosts can
//! still expose durable thread reads through the app-server host bridge.

use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::Duration;

use chrono::DateTime;
use chrono::SecondsFormat;
use chrono::Utc;
use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_protocol::capabilities::SelectedCapabilityRoot;
use codex_protocol::dynamic_tools::DynamicToolSpec;
use codex_protocol::models::BaseInstructions;
use codex_protocol::protocol::InitialHistory;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::ResumedHistory;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::SessionContextWindow;
use codex_protocol::protocol::SessionMeta;
use codex_protocol::protocol::SessionMetaLine;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::ThreadHistoryMode;
use codex_protocol::protocol::ThreadSource;
use codex_state::ThreadMetadataBuilder;
use serde::Deserialize;
use serde::Serialize;

use crate::RolloutConfigView;
use crate::persisted_rollout_items;

type RolloutStore = HashMap<PathBuf, Vec<RolloutItem>>;

static ROLLOUTS: LazyLock<Mutex<RolloutStore>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static THREAD_NAMES: LazyLock<Mutex<HashMap<ThreadId, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn lock_rollouts() -> io::Result<std::sync::MutexGuard<'static, RolloutStore>> {
    ROLLOUTS
        .lock()
        .map_err(|error| io::Error::other(format!("browser rollout store is poisoned: {error}")))
}

fn lock_thread_names() -> io::Result<std::sync::MutexGuard<'static, HashMap<ThreadId, String>>> {
    THREAD_NAMES.lock().map_err(|error| {
        io::Error::other(format!("browser thread-name store is poisoned: {error}"))
    })
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
    pub history_mode: ThreadHistoryMode,
    pub parent_thread_id: Option<ThreadId>,
    pub agent_nickname: Option<String>,
    pub agent_role: Option<String>,
    pub model_provider: Option<String>,
    pub cli_version: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub recency_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadSortKey {
    CreatedAt,
    UpdatedAt,
    RecencyAt,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    offset: usize,
}

impl Serialize for Cursor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.offset.to_string())
    }
}

impl<'de> Deserialize<'de> for Cursor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let offset = value.parse().map_err(serde::de::Error::custom)?;
        Ok(Self { offset })
    }
}

impl From<codex_state::Anchor> for Cursor {
    fn from(_anchor: codex_state::Anchor) -> Self {
        Self { offset: 0 }
    }
}

#[derive(Clone)]
pub struct RolloutRecorder {
    rollout_path: PathBuf,
    history_mode: ThreadHistoryMode,
}

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum RolloutRecorderParams {
    Create {
        session_id: SessionId,
        conversation_id: ThreadId,
        forked_from_id: Option<ThreadId>,
        parent_thread_id: Option<ThreadId>,
        source: Box<SessionSource>,
        thread_source: Option<ThreadSource>,
        originator: String,
        base_instructions: BaseInstructions,
        dynamic_tools: Vec<DynamicToolSpec>,
        selected_capability_roots: Vec<SelectedCapabilityRoot>,
        multi_agent_version: Option<MultiAgentVersion>,
        history_mode: ThreadHistoryMode,
        subagent_history_start_ordinal: Option<u64>,
        initial_window_id: Option<String>,
    },
    Resume {
        path: PathBuf,
    },
}

impl RolloutRecorderParams {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        conversation_id: ThreadId,
        forked_from_id: Option<ThreadId>,
        parent_thread_id: Option<ThreadId>,
        source: SessionSource,
        thread_source: Option<ThreadSource>,
        originator: String,
        base_instructions: BaseInstructions,
        dynamic_tools: Vec<DynamicToolSpec>,
    ) -> Self {
        Self::Create {
            session_id: conversation_id.into(),
            conversation_id,
            forked_from_id,
            parent_thread_id,
            source: Box::new(source),
            thread_source,
            originator,
            base_instructions,
            dynamic_tools,
            selected_capability_roots: Vec::new(),
            multi_agent_version: None,
            history_mode: ThreadHistoryMode::default(),
            subagent_history_start_ordinal: None,
            initial_window_id: None,
        }
    }

    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        if let Self::Create { session_id: id, .. } = &mut self {
            *id = session_id;
        }
        self
    }

    pub fn with_selected_capability_roots(
        mut self,
        selected_capability_roots: Vec<SelectedCapabilityRoot>,
    ) -> Self {
        if let Self::Create {
            selected_capability_roots: roots,
            ..
        } = &mut self
        {
            *roots = selected_capability_roots;
        }
        self
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

    pub fn with_history_mode(mut self, history_mode: ThreadHistoryMode) -> Self {
        if let Self::Create {
            history_mode: mode, ..
        } = &mut self
        {
            *mode = history_mode;
        }
        self
    }

    pub fn with_subagent_history_start_ordinal(
        mut self,
        subagent_history_start_ordinal: Option<u64>,
    ) -> Self {
        if let Self::Create {
            subagent_history_start_ordinal: ordinal,
            ..
        } = &mut self
        {
            *ordinal = subagent_history_start_ordinal;
        }
        self
    }

    pub fn with_initial_window_id(mut self, initial_window_id: String) -> Self {
        if let Self::Create {
            initial_window_id: window_id,
            ..
        } = &mut self
        {
            *window_id = Some(initial_window_id);
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
        _state_db_ctx: Option<StateDbHandle>,
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
        let page = get_threads(
            config.codex_home(),
            page_size,
            cursor,
            sort_key,
            allowed_sources,
            model_providers,
            cwd_filters,
            default_provider,
        )
        .await?;
        Ok(filter_page(page, sort_direction, search_term))
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
        _state_db_ctx: Option<StateDbHandle>,
        _config: &impl RolloutConfigView,
        _page_size: usize,
        _cursor: Option<&Cursor>,
        _sort_key: ThreadSortKey,
        _sort_direction: SortDirection,
        _allowed_sources: &[SessionSource],
        _model_providers: Option<&[String]>,
        _cwd_filters: Option<&[PathBuf]>,
        _default_provider: &str,
        _search_term: Option<&str>,
    ) -> io::Result<ThreadsPage> {
        Ok(ThreadsPage::default())
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
        Self::list_archived_threads(
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
        _state_db_ctx: Option<StateDbHandle>,
        config: &impl RolloutConfigView,
        page_size: usize,
        cursor: Option<&Cursor>,
        sort_key: ThreadSortKey,
        allowed_sources: &[SessionSource],
        model_providers: Option<&[String]>,
        default_provider: &str,
        filter_cwd: Option<&Path>,
    ) -> io::Result<Option<PathBuf>> {
        let cwd_filters = filter_cwd.map(|cwd| vec![cwd.to_path_buf()]);
        Ok(get_threads(
            config.codex_home(),
            page_size,
            cursor,
            sort_key,
            allowed_sources,
            model_providers,
            cwd_filters.as_deref(),
            default_provider,
        )
        .await?
        .items
        .into_iter()
        .next()
        .map(|item| item.path))
    }

    pub async fn new(
        config: &impl RolloutConfigView,
        params: RolloutRecorderParams,
    ) -> io::Result<Self> {
        match params {
            RolloutRecorderParams::Create {
                session_id,
                conversation_id,
                forked_from_id,
                parent_thread_id,
                source,
                thread_source,
                originator,
                base_instructions,
                dynamic_tools,
                selected_capability_roots,
                multi_agent_version,
                history_mode,
                subagent_history_start_ordinal,
                initial_window_id,
            } => {
                let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
                let rollout_path = config
                    .codex_home()
                    .join(crate::SESSIONS_SUBDIR)
                    .join(format!(
                        "browser-rollout-{timestamp}-{conversation_id}.jsonl"
                    ));
                let session_meta = SessionMeta {
                    session_id,
                    id: conversation_id,
                    forked_from_id,
                    parent_thread_id,
                    timestamp,
                    cwd: config.cwd().to_path_buf(),
                    originator,
                    cli_version: env!("CARGO_PKG_VERSION").to_string(),
                    agent_nickname: source.get_nickname(),
                    agent_role: source.get_agent_role(),
                    agent_path: source.get_agent_path().map(Into::into),
                    source: *source,
                    thread_source,
                    model_provider: Some(config.model_provider_id().to_string()),
                    base_instructions: Some(base_instructions),
                    dynamic_tools: (!dynamic_tools.is_empty()).then_some(dynamic_tools),
                    selected_capability_roots,
                    memory_mode: (!config.generate_memories()).then_some("disabled".to_string()),
                    history_mode,
                    history_base: None,
                    subagent_history_start_ordinal,
                    multi_agent_version,
                    context_window: initial_window_id.map(SessionContextWindow::new),
                };
                lock_rollouts()?.insert(
                    rollout_path.clone(),
                    vec![RolloutItem::SessionMeta(SessionMetaLine {
                        meta: session_meta,
                        git: None,
                    })],
                );
                Ok(Self {
                    rollout_path,
                    history_mode,
                })
            }
            RolloutRecorderParams::Resume { path } => {
                let history_mode = lock_rollouts()?
                    .get(&path)
                    .and_then(|items| {
                        items.iter().find_map(|item| match item {
                            RolloutItem::SessionMeta(meta) => Some(meta.meta.history_mode),
                            _ => None,
                        })
                    })
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("browser rollout not found: {}", path.display()),
                        )
                    })?;
                Ok(Self {
                    rollout_path: path,
                    history_mode,
                })
            }
        }
    }

    pub fn rollout_path(&self) -> &Path {
        self.rollout_path.as_path()
    }

    pub async fn record_canonical_items(&self, items: &[RolloutItem]) -> io::Result<()> {
        let persisted = persisted_rollout_items(items, self.history_mode);
        lock_rollouts()?
            .entry(self.rollout_path.clone())
            .or_default()
            .extend(persisted);
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
        let items = lock_rollouts()?.get(path).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("browser rollout not found: {}", path.display()),
            )
        })?;
        let thread_id = items.iter().find_map(|item| match item {
            RolloutItem::SessionMeta(meta) => Some(meta.meta.id),
            _ => None,
        });
        Ok((items, thread_id, 0))
    }

    pub async fn get_rollout_history(path: &Path) -> io::Result<InitialHistory> {
        let (items, thread_id, _) = Self::load_rollout_items(path).await?;
        let conversation_id =
            thread_id.ok_or_else(|| io::Error::other("browser rollout has no session metadata"))?;
        Ok(InitialHistory::Resumed(ResumedHistory {
            conversation_id,
            history: Arc::new(items),
            rollout_path: Some(path.to_path_buf()),
        }))
    }

    pub async fn shutdown(&self) -> io::Result<()> {
        Ok(())
    }
}

fn filter_page(
    mut page: ThreadsPage,
    sort_direction: SortDirection,
    search_term: Option<&str>,
) -> ThreadsPage {
    if let Some(search_term) = search_term {
        let search_term = search_term.to_ascii_lowercase();
        page.items.retain(|item| {
            item.preview
                .as_deref()
                .or(item.first_user_message.as_deref())
                .is_some_and(|value| value.to_ascii_lowercase().contains(&search_term))
        });
    }
    if matches!(sort_direction, SortDirection::Asc) {
        page.items.reverse();
    }
    page
}

fn thread_item(path: PathBuf, items: &[RolloutItem]) -> Option<ThreadItem> {
    let meta = items.iter().find_map(|item| match item {
        RolloutItem::SessionMeta(meta) => Some(meta),
        _ => None,
    })?;
    Some(ThreadItem {
        path,
        thread_id: Some(meta.meta.id),
        cwd: Some(meta.meta.cwd.clone()),
        source: Some(meta.meta.source.clone()),
        history_mode: meta.meta.history_mode,
        parent_thread_id: meta.meta.parent_thread_id,
        agent_nickname: meta.meta.agent_nickname.clone(),
        agent_role: meta.meta.agent_role.clone(),
        model_provider: meta.meta.model_provider.clone(),
        cli_version: Some(meta.meta.cli_version.clone()),
        created_at: Some(meta.meta.timestamp.clone()),
        updated_at: Some(meta.meta.timestamp.clone()),
        recency_at: Some(meta.meta.timestamp.clone()),
        ..Default::default()
    })
}

impl From<codex_state::ThreadsPage> for ThreadsPage {
    fn from(db_page: codex_state::ThreadsPage) -> Self {
        let codex_state::ThreadsPage {
            items,
            parent_thread_ids,
            next_anchor,
            num_scanned_rows,
        } = db_page;
        let items = items
            .into_iter()
            .map(|item| {
                let parent_thread_id = parent_thread_ids.get(&item.id).copied();
                ThreadItem {
                    path: item.rollout_path,
                    thread_id: Some(item.id),
                    first_user_message: item.first_user_message,
                    preview: item.preview,
                    cwd: Some(item.cwd),
                    git_branch: item.git_branch,
                    git_sha: item.git_sha,
                    git_origin_url: item.git_origin_url,
                    source: Some(
                        serde_json::from_str(item.source.as_str())
                            .or_else(|_| {
                                serde_json::from_value(serde_json::Value::String(item.source))
                            })
                            .unwrap_or(SessionSource::Unknown),
                    ),
                    history_mode: item.history_mode,
                    parent_thread_id,
                    agent_nickname: item.agent_nickname,
                    agent_role: item.agent_role,
                    model_provider: Some(item.model_provider),
                    cli_version: Some(item.cli_version),
                    created_at: Some(item.created_at.to_rfc3339_opts(SecondsFormat::Secs, true)),
                    updated_at: Some(item.updated_at.to_rfc3339_opts(SecondsFormat::Millis, true)),
                    recency_at: Some(item.recency_at.to_rfc3339_opts(SecondsFormat::Millis, true)),
                }
            })
            .collect();
        Self {
            items,
            next_cursor: next_anchor.map(Into::into),
            num_scanned_files: num_scanned_rows,
            reached_scan_cap: false,
        }
    }
}

pub async fn get_threads(
    codex_home: &Path,
    page_size: usize,
    cursor: Option<&Cursor>,
    _sort_key: ThreadSortKey,
    allowed_sources: &[SessionSource],
    model_providers: Option<&[String]>,
    cwd_filters: Option<&[PathBuf]>,
    _default_provider: &str,
) -> io::Result<ThreadsPage> {
    let store = lock_rollouts()?;
    let mut items = store
        .iter()
        .filter(|(path, _)| path.starts_with(codex_home))
        .filter_map(|(path, rollout)| thread_item(path.clone(), rollout))
        .filter(|item| {
            allowed_sources.is_empty()
                || item
                    .source
                    .as_ref()
                    .is_some_and(|source| allowed_sources.contains(source))
        })
        .filter(|item| {
            model_providers.is_none_or(|providers| {
                item.model_provider
                    .as_ref()
                    .is_some_and(|provider| providers.contains(provider))
            })
        })
        .filter(|item| {
            cwd_filters.is_none_or(|filters| {
                item.cwd
                    .as_ref()
                    .is_some_and(|cwd| filters.iter().any(|filter| cwd == filter))
            })
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    let num_scanned_files = items.len();
    let offset = cursor.map_or(0, |cursor| cursor.offset).min(items.len());
    let end = offset.saturating_add(page_size).min(items.len());
    let next_cursor = (end < items.len()).then_some(Cursor { offset: end });
    Ok(ThreadsPage {
        items: items.drain(offset..end).collect(),
        next_cursor,
        num_scanned_files,
        reached_scan_cap: false,
    })
}

pub async fn get_threads_in_root(
    root: PathBuf,
    page_size: usize,
    cursor: Option<&Cursor>,
    sort_key: ThreadSortKey,
    config: ThreadListConfig<'_>,
) -> io::Result<ThreadsPage> {
    get_threads(
        &root,
        page_size,
        cursor,
        sort_key,
        config.allowed_sources,
        config.model_providers,
        config.cwd_filters,
        config.default_provider,
    )
    .await
}

pub fn parse_cursor(token: &str) -> Option<Cursor> {
    token.parse().ok().map(|offset| Cursor { offset })
}

pub async fn read_thread_item_from_rollout(path: PathBuf) -> Option<ThreadItem> {
    lock_rollouts()
        .ok()
        .and_then(|store| store.get(&path).and_then(|items| thread_item(path, items)))
}

pub async fn read_head_for_summary(path: &Path) -> io::Result<Vec<serde_json::Value>> {
    Ok(lock_rollouts()?
        .get(path)
        .into_iter()
        .flatten()
        .take(10)
        .filter_map(|item| serde_json::to_value(item).ok())
        .collect())
}

pub async fn read_session_meta_line(path: &Path) -> io::Result<SessionMetaLine> {
    lock_rollouts()?
        .get(path)
        .and_then(|items| {
            items.iter().find_map(|item| match item {
                RolloutItem::SessionMeta(meta) => Some(meta.clone()),
                _ => None,
            })
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("browser rollout metadata not found: {}", path.display()),
            )
        })
}

pub async fn find_thread_path_by_id_str(
    codex_home: &Path,
    id_str: &str,
    _state_db_ctx: Option<&codex_state::StateRuntime>,
) -> io::Result<Option<PathBuf>> {
    let Ok(id) = ThreadId::from_string(id_str) else {
        return Ok(None);
    };
    Ok(lock_rollouts()?.iter().find_map(|(path, items)| {
        (path.starts_with(codex_home)
            && items
                .iter()
                .any(|item| matches!(item, RolloutItem::SessionMeta(meta) if meta.meta.id == id)))
        .then(|| path.clone())
    }))
}

pub async fn find_archived_thread_path_by_id_str(
    _codex_home: &Path,
    _id_str: &str,
    _state_db_ctx: Option<&codex_state::StateRuntime>,
) -> io::Result<Option<PathBuf>> {
    Ok(None)
}

pub async fn append_thread_name(
    _codex_home: &Path,
    thread_id: ThreadId,
    name: &str,
) -> io::Result<()> {
    lock_thread_names()?.insert(thread_id, name.to_string());
    Ok(())
}

pub async fn remove_thread_name_entries(_codex_home: &Path, thread_id: ThreadId) -> io::Result<()> {
    lock_thread_names()?.remove(&thread_id);
    Ok(())
}

pub async fn find_thread_name_by_id(
    _codex_home: &Path,
    id: &ThreadId,
) -> io::Result<Option<String>> {
    Ok(lock_thread_names()?.get(id).cloned())
}

pub async fn find_thread_names_by_ids(
    _codex_home: &Path,
    ids: &HashSet<ThreadId>,
) -> io::Result<HashMap<ThreadId, String>> {
    Ok(lock_thread_names()?
        .iter()
        .filter(|(id, _)| ids.contains(id))
        .map(|(id, name)| (*id, name.clone()))
        .collect())
}

pub async fn find_thread_meta_by_name_str(
    codex_home: &Path,
    name: &str,
    state_db_ctx: Option<&codex_state::StateRuntime>,
) -> io::Result<Option<(PathBuf, SessionMetaLine)>> {
    let id = lock_thread_names()?
        .iter()
        .find_map(|(id, stored_name)| (stored_name == name).then_some(*id));
    let Some(id) = id else {
        return Ok(None);
    };
    let Some(path) = find_thread_path_by_id_str(codex_home, &id.to_string(), state_db_ctx).await?
    else {
        return Ok(None);
    };
    Ok(Some((path.clone(), read_session_meta_line(&path).await?)))
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
    items: &[RolloutItem],
    rollout_path: &Path,
) -> Option<ThreadMetadataBuilder> {
    let meta = items.iter().find_map(|item| match item {
        RolloutItem::SessionMeta(meta) => Some(meta),
        _ => None,
    })?;
    let created_at = DateTime::parse_from_rfc3339(&meta.meta.timestamp)
        .ok()?
        .with_timezone(&Utc);
    let mut builder = ThreadMetadataBuilder::new(
        meta.meta.id,
        rollout_path.to_path_buf(),
        created_at,
        meta.meta.source.clone(),
    );
    builder.history_mode = meta.meta.history_mode;
    builder.model_provider = meta.meta.model_provider.clone();
    builder.agent_nickname = meta.meta.agent_nickname.clone();
    builder.agent_role = meta.meta.agent_role.clone();
    builder.agent_path = meta.meta.agent_path.clone();
    builder.cwd = meta.meta.cwd.clone();
    builder.cli_version = Some(meta.meta.cli_version.clone());
    Some(builder)
}

pub async fn append_rollout_item_to_path(
    rollout_path: &Path,
    item: &RolloutItem,
) -> io::Result<()> {
    lock_rollouts()?
        .entry(rollout_path.to_path_buf())
        .or_default()
        .push(item.clone());
    Ok(())
}

pub fn spawn_rollout_compression_worker(_codex_home: PathBuf) {}

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
    lock_rollouts()
        .ok()
        .and_then(|store| store.contains_key(path).then(|| path.to_path_buf()))
}

pub fn plain_rollout_path(path: &Path) -> PathBuf {
    path.to_path_buf()
}

pub type RolloutSearchMatches = HashMap<PathBuf, Option<String>>;

pub async fn search_rollout_paths(
    _rg_command: &Path,
    codex_home: &Path,
    archived: bool,
    search_term: &str,
) -> io::Result<HashSet<PathBuf>> {
    Ok(
        search_rollout_matches(Path::new("rg"), codex_home, archived, search_term)
            .await?
            .into_keys()
            .collect(),
    )
}

pub async fn search_rollout_matches(
    _rg_command: &Path,
    codex_home: &Path,
    archived: bool,
    search_term: &str,
) -> io::Result<RolloutSearchMatches> {
    if archived {
        return Ok(HashMap::new());
    }
    let needle = search_term.to_ascii_lowercase();
    Ok(lock_rollouts()?
        .iter()
        .filter(|(path, _)| path.starts_with(codex_home))
        .filter_map(|(path, items)| {
            let text = serde_json::to_string(items).ok()?;
            text.to_ascii_lowercase()
                .contains(&needle)
                .then(|| (path.clone(), Some(text)))
        })
        .collect())
}

pub async fn first_rollout_content_match_snippet(
    path: &Path,
    search_term: &str,
) -> io::Result<Option<String>> {
    let needle = search_term.to_ascii_lowercase();
    Ok(lock_rollouts()?.get(path).and_then(|items| {
        serde_json::to_string(items)
            .ok()
            .filter(|text| text.to_ascii_lowercase().contains(&needle))
    }))
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
        anyhow::bail!("browser Codex uses the app-server host storage bridge instead of SQLite")
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
        _context: Option<&codex_state::StateRuntime>,
        _codex_home: &Path,
        _page_size: usize,
        _cursor: Option<&Cursor>,
        _sort_key: ThreadSortKey,
        _allowed_sources: &[SessionSource],
        _model_providers: Option<&[String]>,
        _archived_only: bool,
        _stage: &str,
    ) -> Option<Vec<ThreadId>> {
        None
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_threads_db(
        _context: Option<&codex_state::StateRuntime>,
        _codex_home: &Path,
        _page_size: usize,
        _cursor: Option<&Cursor>,
        _sort_key: ThreadSortKey,
        _sort_direction: SortDirection,
        _allowed_sources: &[SessionSource],
        _model_providers: Option<&[String]>,
        _cwd_filters: Option<&[PathBuf]>,
        _relation_filter: Option<codex_state::ThreadRelationFilter>,
        _archived: bool,
        _search_term: Option<&str>,
    ) -> Option<codex_state::ThreadsPage> {
        None
    }

    pub async fn find_rollout_path_by_id(
        _context: Option<&codex_state::StateRuntime>,
        thread_id: ThreadId,
        archived_only: Option<bool>,
        _stage: &str,
    ) -> Option<PathBuf> {
        if archived_only == Some(true) {
            return None;
        }
        let store = lock_rollouts().ok()?;
        store.iter().find_map(|(path, items)| {
            items
                .iter()
                .any(|item| {
                    matches!(item, RolloutItem::SessionMeta(meta) if meta.meta.id == thread_id)
                })
                .then(|| path.clone())
        })
    }

    pub async fn mark_thread_memory_mode_polluted(
        _context: Option<&codex_state::StateRuntime>,
        _thread_id: ThreadId,
        _stage: &str,
    ) {
    }

    pub async fn reconcile_rollout(
        _context: Option<&codex_state::StateRuntime>,
        _rollout_path: &Path,
        _default_provider: &str,
        _builder: Option<&ThreadMetadataBuilder>,
        _items: &[RolloutItem],
        _archived_only: Option<bool>,
        _new_thread_memory_mode: Option<&str>,
    ) {
    }

    pub async fn read_repair_rollout_path(
        _context: Option<&codex_state::StateRuntime>,
        _thread_id: Option<ThreadId>,
        _archived_only: Option<bool>,
        _rollout_path: &Path,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn apply_rollout_items(
        _context: Option<&codex_state::StateRuntime>,
        _rollout_path: &Path,
        _default_provider: &str,
        _builder: Option<&ThreadMetadataBuilder>,
        _items: &[RolloutItem],
        _stage: &str,
        _new_thread_memory_mode: Option<&str>,
        _updated_at_override: Option<DateTime<Utc>>,
    ) {
    }

    pub async fn touch_thread_updated_at(
        _context: Option<&codex_state::StateRuntime>,
        _thread_id: Option<ThreadId>,
        _updated_at: DateTime<Utc>,
        _stage: &str,
    ) -> bool {
        false
    }

    struct NoopDbTelemetry;

    impl codex_state::DbTelemetry for NoopDbTelemetry {
        fn counter(&self, _name: &str, _inc: i64, _tags: &[(&str, &str)]) {}

        fn record_duration(&self, _name: &str, _duration: Duration, _tags: &[(&str, &str)]) {}
    }
}

pub use state_db::sqlite_telemetry_recorder;
