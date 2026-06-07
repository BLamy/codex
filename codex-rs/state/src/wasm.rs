use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use codex_protocol::ThreadId;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::protocol::SessionMetaLine;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::ThreadSource;
use codex_protocol::protocol::TurnContextItem;
use codex_protocol::protocol::USER_MESSAGE_BEGIN;
use codex_protocol::protocol::UserMessageEvent;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;
use strum::AsRefStr;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

const IMAGE_ONLY_USER_MESSAGE_PLACEHOLDER: &str = "[Image]";

fn unsupported<T>(operation: &str) -> anyhow::Result<T> {
    Err(anyhow::anyhow!(
        "codex-state operation `{operation}` requires a browser persistence host shim"
    ))
}

fn lock_err() -> anyhow::Error {
    anyhow::anyhow!("codex-state wasm in-memory store lock poisoned")
}

#[derive(Clone, Debug, Serialize)]
pub struct LogEntry {
    pub ts: i64,
    pub ts_nanos: i64,
    pub level: String,
    pub target: String,
    pub message: Option<String>,
    pub feedback_log_body: Option<String>,
    pub thread_id: Option<String>,
    pub process_uuid: Option<String>,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct LogRow {
    pub id: i64,
    pub ts: i64,
    pub ts_nanos: i64,
    pub level: String,
    pub target: String,
    pub message: Option<String>,
    pub thread_id: Option<String>,
    pub process_uuid: Option<String>,
    pub file: Option<String>,
    pub line: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct LogQuery {
    pub levels_upper: Vec<String>,
    pub from_ts: Option<i64>,
    pub to_ts: Option<i64>,
    pub module_like: Vec<String>,
    pub file_like: Vec<String>,
    pub thread_ids: Vec<String>,
    pub search: Option<String>,
    pub include_threadless: bool,
    pub after_id: Option<i64>,
    pub limit: Option<usize>,
    pub descending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl AgentJobStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(anyhow::anyhow!("invalid agent job status: {value}")),
        }
    }

    pub fn is_final(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentJobItemStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl AgentJobItemStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(anyhow::anyhow!("invalid agent job item status: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentJob {
    pub id: String,
    pub name: String,
    pub status: AgentJobStatus,
    pub instruction: String,
    pub auto_export: bool,
    pub max_runtime_seconds: Option<u64>,
    pub output_schema_json: Option<Value>,
    pub input_headers: Vec<String>,
    pub input_csv_path: String,
    pub output_csv_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentJobItem {
    pub job_id: String,
    pub item_id: String,
    pub row_index: i64,
    pub source_id: Option<String>,
    pub row_json: Value,
    pub status: AgentJobItemStatus,
    pub assigned_thread_id: Option<String>,
    pub attempt_count: i64,
    pub result_json: Option<Value>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub reported_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentJobProgress {
    pub total_items: usize,
    pub pending_items: usize,
    pub running_items: usize,
    pub completed_items: usize,
    pub failed_items: usize,
}

#[derive(Debug, Clone)]
pub struct AgentJobCreateParams {
    pub id: String,
    pub name: String,
    pub instruction: String,
    pub auto_export: bool,
    pub max_runtime_seconds: Option<u64>,
    pub output_schema_json: Option<Value>,
    pub input_headers: Vec<String>,
    pub input_csv_path: String,
    pub output_csv_path: String,
}

#[derive(Debug, Clone)]
pub struct AgentJobItemCreateParams {
    pub item_id: String,
    pub row_index: i64,
    pub source_id: Option<String>,
    pub row_json: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackfillState {
    pub status: BackfillStatus,
    pub last_watermark: Option<String>,
    pub last_success_at: Option<DateTime<Utc>>,
}

impl Default for BackfillState {
    fn default() -> Self {
        Self {
            status: BackfillStatus::Pending,
            last_watermark: None,
            last_success_at: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackfillStatus {
    Pending,
    Running,
    Complete,
}

impl BackfillStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Complete => "complete",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "complete" => Ok(Self::Complete),
            _ => Err(anyhow::anyhow!("invalid backfill status: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum DirectionalThreadSpawnEdgeStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage1Output {
    pub thread_id: ThreadId,
    pub rollout_path: PathBuf,
    pub source_updated_at: DateTime<Utc>,
    pub raw_memory: String,
    pub rollout_summary: String,
    pub rollout_slug: Option<String>,
    pub cwd: PathBuf,
    pub git_branch: Option<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stage1JobClaimOutcome {
    Claimed { ownership_token: String },
    SkippedUpToDate,
    SkippedRunning,
    SkippedRetryBackoff,
    SkippedRetryExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage1JobClaim {
    pub thread: ThreadMetadata,
    pub ownership_token: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Stage1StartupClaimParams<'a> {
    pub scan_limit: usize,
    pub max_claimed: usize,
    pub max_age_days: i64,
    pub min_rollout_idle_hours: i64,
    pub allowed_sources: &'a [String],
    pub lease_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase2JobClaimOutcome {
    Claimed {
        ownership_token: String,
        input_watermark: i64,
    },
    SkippedRetryUnavailable,
    SkippedCooldown,
    SkippedRunning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    pub ts: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadsPage {
    pub items: Vec<ThreadMetadata>,
    pub next_anchor: Option<Anchor>,
    pub num_scanned_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractionOutcome {
    pub metadata: ThreadMetadata,
    pub memory_mode: Option<String>,
    pub parse_errors: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadMetadata {
    pub id: ThreadId,
    pub rollout_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: String,
    pub thread_source: Option<ThreadSource>,
    pub agent_nickname: Option<String>,
    pub agent_role: Option<String>,
    pub agent_path: Option<String>,
    pub model_provider: String,
    pub model: Option<String>,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub cwd: PathBuf,
    pub cli_version: String,
    pub title: String,
    pub preview: Option<String>,
    pub sandbox_policy: String,
    pub approval_mode: String,
    pub tokens_used: i64,
    pub first_user_message: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub git_sha: Option<String>,
    pub git_branch: Option<String>,
    pub git_origin_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadMetadataBuilder {
    pub id: ThreadId,
    pub rollout_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub source: SessionSource,
    pub thread_source: Option<ThreadSource>,
    pub agent_nickname: Option<String>,
    pub agent_role: Option<String>,
    pub agent_path: Option<String>,
    pub model_provider: Option<String>,
    pub cwd: PathBuf,
    pub cli_version: Option<String>,
    pub sandbox_policy: SandboxPolicy,
    pub approval_mode: AskForApproval,
    pub archived_at: Option<DateTime<Utc>>,
    pub git_sha: Option<String>,
    pub git_branch: Option<String>,
    pub git_origin_url: Option<String>,
}

impl ThreadMetadataBuilder {
    pub fn new(
        id: ThreadId,
        rollout_path: PathBuf,
        created_at: DateTime<Utc>,
        source: SessionSource,
    ) -> Self {
        Self {
            id,
            rollout_path,
            created_at,
            updated_at: None,
            source,
            thread_source: None,
            agent_nickname: None,
            agent_role: None,
            agent_path: None,
            model_provider: None,
            cwd: PathBuf::new(),
            cli_version: None,
            sandbox_policy: SandboxPolicy::new_read_only_policy(),
            approval_mode: AskForApproval::OnRequest,
            archived_at: None,
            git_sha: None,
            git_branch: None,
            git_origin_url: None,
        }
    }

    pub fn build(&self, default_provider: &str) -> ThreadMetadata {
        let created_at = canonicalize_datetime(self.created_at);
        let updated_at = self
            .updated_at
            .map(canonicalize_datetime)
            .unwrap_or(created_at);
        ThreadMetadata {
            id: self.id,
            rollout_path: self.rollout_path.clone(),
            created_at,
            updated_at,
            source: enum_to_string(&self.source),
            thread_source: self.thread_source,
            agent_nickname: self.agent_nickname.clone(),
            agent_role: self.agent_role.clone(),
            agent_path: self
                .agent_path
                .clone()
                .or_else(|| self.source.get_agent_path().map(Into::into)),
            model_provider: self
                .model_provider
                .clone()
                .unwrap_or_else(|| default_provider.to_string()),
            model: None,
            reasoning_effort: None,
            cwd: self.cwd.clone(),
            cli_version: self.cli_version.clone().unwrap_or_default(),
            title: String::new(),
            preview: None,
            sandbox_policy: enum_to_string(&self.sandbox_policy),
            approval_mode: enum_to_string(&self.approval_mode),
            tokens_used: 0,
            first_user_message: None,
            archived_at: self.archived_at.map(canonicalize_datetime),
            git_sha: self.git_sha.clone(),
            git_branch: self.git_branch.clone(),
            git_origin_url: self.git_origin_url.clone(),
        }
    }
}

impl ThreadMetadata {
    pub fn prefer_existing_git_info(&mut self, existing: &Self) {
        if existing.git_sha.is_some() {
            self.git_sha = existing.git_sha.clone();
        }
        if existing.git_branch.is_some() {
            self.git_branch = existing.git_branch.clone();
        }
        if existing.git_origin_url.is_some() {
            self.git_origin_url = existing.git_origin_url.clone();
        }
    }

    pub fn prefer_existing_explicit_title(&mut self, existing: &Self) {
        let existing_title = existing.title.trim();
        if existing_title.is_empty()
            || existing.first_user_message.as_deref().map(str::trim) == Some(existing_title)
        {
            return;
        }
        let title = self.title.trim();
        if title.is_empty() || self.first_user_message.as_deref().map(str::trim) == Some(title) {
            self.title = existing.title.clone();
        }
    }

    pub fn diff_fields(&self, other: &Self) -> Vec<&'static str> {
        let mut diffs = Vec::new();
        if self.id != other.id {
            diffs.push("id");
        }
        if self.rollout_path != other.rollout_path {
            diffs.push("rollout_path");
        }
        if self.created_at != other.created_at {
            diffs.push("created_at");
        }
        if self.updated_at != other.updated_at {
            diffs.push("updated_at");
        }
        if self.source != other.source {
            diffs.push("source");
        }
        if self.thread_source != other.thread_source {
            diffs.push("thread_source");
        }
        if self.agent_nickname != other.agent_nickname {
            diffs.push("agent_nickname");
        }
        if self.agent_role != other.agent_role {
            diffs.push("agent_role");
        }
        if self.agent_path != other.agent_path {
            diffs.push("agent_path");
        }
        if self.model_provider != other.model_provider {
            diffs.push("model_provider");
        }
        if self.model != other.model {
            diffs.push("model");
        }
        if self.reasoning_effort != other.reasoning_effort {
            diffs.push("reasoning_effort");
        }
        if self.cwd != other.cwd {
            diffs.push("cwd");
        }
        if self.cli_version != other.cli_version {
            diffs.push("cli_version");
        }
        if self.title != other.title {
            diffs.push("title");
        }
        if self.preview != other.preview {
            diffs.push("preview");
        }
        if self.sandbox_policy != other.sandbox_policy {
            diffs.push("sandbox_policy");
        }
        if self.approval_mode != other.approval_mode {
            diffs.push("approval_mode");
        }
        if self.tokens_used != other.tokens_used {
            diffs.push("tokens_used");
        }
        if self.first_user_message != other.first_user_message {
            diffs.push("first_user_message");
        }
        if self.archived_at != other.archived_at {
            diffs.push("archived_at");
        }
        if self.git_sha != other.git_sha {
            diffs.push("git_sha");
        }
        if self.git_branch != other.git_branch {
            diffs.push("git_branch");
        }
        if self.git_origin_url != other.git_origin_url {
            diffs.push("git_origin_url");
        }
        diffs
    }
}

fn canonicalize_datetime(dt: DateTime<Utc>) -> DateTime<Utc> {
    epoch_millis_to_datetime(datetime_to_epoch_millis(dt)).unwrap_or(dt)
}

fn datetime_to_epoch_millis(dt: DateTime<Utc>) -> i64 {
    dt.timestamp_millis()
}

fn epoch_millis_to_datetime(value: i64) -> Result<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp_millis(value)
        .ok_or_else(|| anyhow::anyhow!("invalid unix timestamp millis: {value}"))
}

fn anchor_from_item(item: &ThreadMetadata, sort_key: SortKey) -> Anchor {
    let ts = match sort_key {
        SortKey::CreatedAt => item.created_at,
        SortKey::UpdatedAt => item.updated_at,
    };
    Anchor { ts }
}

#[derive(Debug, Clone)]
pub struct BackfillStats {
    pub scanned: usize,
    pub upserted: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadGoalStatus {
    Active,
    Paused,
    Blocked,
    UsageLimited,
    BudgetLimited,
    Complete,
}

impl ThreadGoalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Blocked => "blocked",
            Self::UsageLimited => "usage_limited",
            Self::BudgetLimited => "budget_limited",
            Self::Complete => "complete",
        }
    }

    pub fn is_active(self) -> bool {
        self == Self::Active
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::BudgetLimited | Self::Complete)
    }
}

impl TryFrom<&str> for ThreadGoalStatus {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "active" => Ok(Self::Active),
            "paused" => Ok(Self::Paused),
            "blocked" => Ok(Self::Blocked),
            "usage_limited" => Ok(Self::UsageLimited),
            "budget_limited" => Ok(Self::BudgetLimited),
            "complete" => Ok(Self::Complete),
            other => Err(anyhow::anyhow!("unknown thread goal status `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadGoal {
    pub thread_id: ThreadId,
    pub goal_id: String,
    pub objective: String,
    pub status: ThreadGoalStatus,
    pub token_budget: Option<i64>,
    pub tokens_used: i64,
    pub time_used_seconds: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn apply_rollout_item(
    metadata: &mut ThreadMetadata,
    item: &RolloutItem,
    default_provider: &str,
) {
    match item {
        RolloutItem::SessionMeta(meta_line) => apply_session_meta_from_item(metadata, meta_line),
        RolloutItem::TurnContext(turn_ctx) => apply_turn_context(metadata, turn_ctx),
        RolloutItem::EventMsg(event) => apply_event_msg(metadata, event),
        RolloutItem::ResponseItem(item) => apply_response_item(metadata, item),
        RolloutItem::Compacted(_) => {}
    }
    if metadata.model_provider.is_empty() {
        metadata.model_provider = default_provider.to_string();
    }
}

pub fn rollout_item_affects_thread_metadata(item: &RolloutItem) -> bool {
    match item {
        RolloutItem::SessionMeta(_) | RolloutItem::TurnContext(_) => true,
        RolloutItem::EventMsg(
            EventMsg::TokenCount(_) | EventMsg::UserMessage(_) | EventMsg::ThreadGoalUpdated(_),
        ) => true,
        RolloutItem::EventMsg(_) | RolloutItem::ResponseItem(_) | RolloutItem::Compacted(_) => {
            false
        }
    }
}

fn apply_session_meta_from_item(metadata: &mut ThreadMetadata, meta_line: &SessionMetaLine) {
    if metadata.id != meta_line.meta.id {
        return;
    }
    metadata.id = meta_line.meta.id;
    metadata.source = enum_to_string(&meta_line.meta.source);
    metadata.thread_source = meta_line.meta.thread_source;
    metadata.agent_nickname = meta_line.meta.agent_nickname.clone();
    metadata.agent_role = meta_line.meta.agent_role.clone();
    metadata.agent_path = meta_line.meta.agent_path.clone();
    if let Some(provider) = meta_line.meta.model_provider.as_deref() {
        metadata.model_provider = provider.to_string();
    }
    if !meta_line.meta.cli_version.is_empty() {
        metadata.cli_version = meta_line.meta.cli_version.clone();
    }
    if !meta_line.meta.cwd.as_os_str().is_empty() {
        metadata.cwd = meta_line.meta.cwd.clone();
    }
    if let Some(git) = meta_line.git.as_ref() {
        metadata.git_sha = git.commit_hash.as_ref().map(|sha| sha.0.clone());
        metadata.git_branch = git.branch.clone();
        metadata.git_origin_url = git.repository_url.clone();
    }
}

fn apply_turn_context(metadata: &mut ThreadMetadata, turn_ctx: &TurnContextItem) {
    if metadata.cwd.as_os_str().is_empty() {
        metadata.cwd = turn_ctx.cwd.clone();
    }
    metadata.model = Some(turn_ctx.model.clone());
    metadata.reasoning_effort = turn_ctx.effort.clone();
    metadata.sandbox_policy =
        serde_json::to_string(&turn_ctx.permission_profile()).unwrap_or_default();
    metadata.approval_mode = enum_to_string(&turn_ctx.approval_policy);
}

fn apply_event_msg(metadata: &mut ThreadMetadata, event: &EventMsg) {
    match event {
        EventMsg::TokenCount(token_count) => {
            if let Some(info) = token_count.info.as_ref() {
                metadata.tokens_used = info.total_token_usage.total_tokens.max(0);
            }
        }
        EventMsg::UserMessage(user) => {
            let preview = user_message_preview(user);
            if metadata.first_user_message.is_none() {
                metadata.first_user_message = preview.clone();
            }
            if metadata.preview.is_none() {
                metadata.preview = preview;
            }
            if metadata.title.is_empty() {
                let title = strip_user_message_prefix(user.message.as_str());
                if !title.is_empty() {
                    metadata.title = title.to_string();
                }
            }
        }
        EventMsg::ThreadGoalUpdated(event) => {
            let objective = event.goal.objective.trim();
            if !objective.is_empty() && metadata.preview.is_none() {
                metadata.preview = Some(objective.to_string());
            }
        }
        _ => {}
    }
}

fn apply_response_item(_metadata: &mut ThreadMetadata, _item: &ResponseItem) {}

fn strip_user_message_prefix(text: &str) -> &str {
    match text.find(USER_MESSAGE_BEGIN) {
        Some(idx) => text[idx + USER_MESSAGE_BEGIN.len()..].trim(),
        None => text.trim(),
    }
}

fn user_message_preview(user: &UserMessageEvent) -> Option<String> {
    let message = strip_user_message_prefix(user.message.as_str());
    if !message.is_empty() {
        return Some(message.to_string());
    }
    if user
        .images
        .as_ref()
        .is_some_and(|images| !images.is_empty())
        || !user.local_images.is_empty()
    {
        return Some(IMAGE_ONLY_USER_MESSAGE_PLACEHOLDER.to_string());
    }
    None
}

fn enum_to_string<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(Value::String(s)) => s,
        Ok(other) => other.to_string(),
        Err(_) => String::new(),
    }
}

#[derive(Clone)]
pub struct StateRuntime {
    codex_home: PathBuf,
    default_provider: String,
    thread_goals: GoalStore,
    memories: MemoryStore,
    threads: Arc<Mutex<HashMap<String, ThreadMetadata>>>,
    thread_memory_modes: Arc<Mutex<HashMap<String, String>>>,
    spawn_edges: Arc<Mutex<HashMap<String, (String, DirectionalThreadSpawnEdgeStatus)>>>,
    remote_control_enrollments: Arc<Mutex<HashMap<String, RemoteControlEnrollmentRecord>>>,
}

impl StateRuntime {
    pub async fn init(codex_home: PathBuf, default_provider: String) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            codex_home,
            default_provider,
            thread_goals: GoalStore::default(),
            memories: MemoryStore,
            threads: Arc::new(Mutex::new(HashMap::new())),
            thread_memory_modes: Arc::new(Mutex::new(HashMap::new())),
            spawn_edges: Arc::new(Mutex::new(HashMap::new())),
            remote_control_enrollments: Arc::new(Mutex::new(HashMap::new())),
        }))
    }

    pub fn codex_home(&self) -> &Path {
        self.codex_home.as_path()
    }

    pub fn thread_goals(&self) -> &GoalStore {
        &self.thread_goals
    }

    pub fn memories(&self) -> &MemoryStore {
        &self.memories
    }

    pub async fn clear_memory_data_in_sqlite_home(_sqlite_home: &Path) -> anyhow::Result<bool> {
        Ok(false)
    }

    pub async fn get_backfill_state(&self) -> anyhow::Result<BackfillState> {
        Ok(BackfillState::default())
    }

    pub async fn try_claim_backfill(&self, _lease_seconds: i64) -> anyhow::Result<bool> {
        Ok(false)
    }

    pub async fn mark_backfill_running(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn checkpoint_backfill(&self, _watermark: &str) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn mark_backfill_complete(
        &self,
        _last_watermark: Option<&str>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn get_thread(&self, id: ThreadId) -> anyhow::Result<Option<ThreadMetadata>> {
        Ok(self
            .threads
            .lock()
            .map_err(|_| lock_err())?
            .get(&id.to_string())
            .cloned())
    }

    pub async fn get_thread_memory_mode(&self, id: ThreadId) -> anyhow::Result<Option<String>> {
        Ok(self
            .thread_memory_modes
            .lock()
            .map_err(|_| lock_err())?
            .get(&id.to_string())
            .cloned())
    }

    pub async fn set_thread_preview_if_empty(
        &self,
        thread_id: ThreadId,
        preview: &str,
    ) -> anyhow::Result<bool> {
        let preview = preview.trim();
        if preview.is_empty() {
            return Ok(false);
        }
        let mut threads = self.threads.lock().map_err(|_| lock_err())?;
        let Some(thread) = threads.get_mut(&thread_id.to_string()) else {
            return Ok(false);
        };
        if thread.preview.is_some() {
            return Ok(false);
        }
        thread.preview = Some(preview.to_string());
        Ok(true)
    }

    pub async fn upsert_thread_spawn_edge(
        &self,
        parent_thread_id: ThreadId,
        child_thread_id: ThreadId,
        status: DirectionalThreadSpawnEdgeStatus,
    ) -> anyhow::Result<()> {
        self.spawn_edges.lock().map_err(|_| lock_err())?.insert(
            child_thread_id.to_string(),
            (parent_thread_id.to_string(), status),
        );
        Ok(())
    }

    pub async fn set_thread_spawn_edge_status(
        &self,
        child_thread_id: ThreadId,
        status: DirectionalThreadSpawnEdgeStatus,
    ) -> anyhow::Result<()> {
        if let Some((_, current_status)) = self
            .spawn_edges
            .lock()
            .map_err(|_| lock_err())?
            .get_mut(&child_thread_id.to_string())
        {
            *current_status = status;
        }
        Ok(())
    }

    pub async fn list_thread_spawn_children_with_status(
        &self,
        parent_thread_id: ThreadId,
        status: DirectionalThreadSpawnEdgeStatus,
    ) -> anyhow::Result<Vec<ThreadId>> {
        self.list_thread_spawn_children_matching(parent_thread_id, Some(status))
    }

    pub async fn list_thread_spawn_children(
        &self,
        parent_thread_id: ThreadId,
    ) -> anyhow::Result<Vec<ThreadId>> {
        self.list_thread_spawn_children_matching(parent_thread_id, None)
    }

    pub async fn list_thread_spawn_descendants_with_status(
        &self,
        root_thread_id: ThreadId,
        status: DirectionalThreadSpawnEdgeStatus,
    ) -> anyhow::Result<Vec<ThreadId>> {
        self.list_thread_spawn_descendants_matching(root_thread_id, Some(status))
    }

    pub async fn list_thread_spawn_descendants(
        &self,
        root_thread_id: ThreadId,
    ) -> anyhow::Result<Vec<ThreadId>> {
        self.list_thread_spawn_descendants_matching(root_thread_id, None)
    }

    pub async fn find_thread_spawn_child_by_path(
        &self,
        parent_thread_id: ThreadId,
        agent_path: &str,
    ) -> anyhow::Result<Option<ThreadId>> {
        let children = self.list_thread_spawn_children(parent_thread_id).await?;
        self.find_thread_by_path(children, agent_path)
    }

    pub async fn find_thread_spawn_descendant_by_path(
        &self,
        root_thread_id: ThreadId,
        agent_path: &str,
    ) -> anyhow::Result<Option<ThreadId>> {
        let descendants = self.list_thread_spawn_descendants(root_thread_id).await?;
        self.find_thread_by_path(descendants, agent_path)
    }

    pub async fn find_rollout_path_by_id(&self, id: ThreadId) -> anyhow::Result<Option<PathBuf>> {
        Ok(self.get_thread(id).await?.map(|thread| thread.rollout_path))
    }

    pub async fn find_thread_by_exact_title(
        &self,
        title: &str,
    ) -> anyhow::Result<Option<ThreadMetadata>> {
        Ok(self
            .threads
            .lock()
            .map_err(|_| lock_err())?
            .values()
            .find(|thread| thread.title == title)
            .cloned())
    }

    pub async fn list_threads(
        &self,
        page_size: usize,
        filters: ThreadFilterOptions<'_>,
    ) -> anyhow::Result<ThreadsPage> {
        let mut items: Vec<_> = self
            .threads
            .lock()
            .map_err(|_| lock_err())?
            .values()
            .filter(|thread| filters.archived_only == thread.archived_at.is_some())
            .filter(|thread| {
                filters.allowed_sources.is_empty()
                    || filters
                        .allowed_sources
                        .iter()
                        .any(|source| source == &thread.source)
            })
            .filter(|thread| {
                filters
                    .model_providers
                    .is_none_or(|providers| providers.iter().any(|p| p == &thread.model_provider))
            })
            .filter(|thread| {
                filters
                    .cwd_filters
                    .is_none_or(|filters| filters.iter().any(|cwd| cwd == &thread.cwd))
            })
            .filter(|thread| {
                filters.search_term.is_none_or(|term| {
                    let term = term.to_ascii_lowercase();
                    thread.title.to_ascii_lowercase().contains(term.as_str())
                        || thread
                            .preview
                            .as_deref()
                            .unwrap_or_default()
                            .to_ascii_lowercase()
                            .contains(term.as_str())
                })
            })
            .cloned()
            .collect();
        items.sort_by_key(|thread| match filters.sort_key {
            SortKey::CreatedAt => thread.created_at,
            SortKey::UpdatedAt => thread.updated_at,
        });
        if filters.sort_direction == SortDirection::Desc {
            items.reverse();
        }
        if let Some(anchor) = filters.anchor {
            items.retain(|thread| match (filters.sort_key, filters.sort_direction) {
                (SortKey::CreatedAt, SortDirection::Asc) => thread.created_at > anchor.ts,
                (SortKey::CreatedAt, SortDirection::Desc) => thread.created_at < anchor.ts,
                (SortKey::UpdatedAt, SortDirection::Asc) => thread.updated_at > anchor.ts,
                (SortKey::UpdatedAt, SortDirection::Desc) => thread.updated_at < anchor.ts,
            });
        }
        let num_scanned_rows = items.len();
        let next_anchor = if items.len() > page_size {
            items.truncate(page_size);
            items
                .last()
                .map(|item| anchor_from_item(item, filters.sort_key))
        } else {
            None
        };
        Ok(ThreadsPage {
            items,
            next_anchor,
            num_scanned_rows,
        })
    }

    pub async fn list_thread_ids(
        &self,
        limit: usize,
        anchor: Option<&Anchor>,
        sort_key: SortKey,
        allowed_sources: &[String],
        model_providers: Option<&[String]>,
        archived_only: bool,
    ) -> anyhow::Result<Vec<ThreadId>> {
        let page = self
            .list_threads(
                limit,
                ThreadFilterOptions {
                    archived_only,
                    allowed_sources,
                    model_providers,
                    cwd_filters: None,
                    anchor,
                    sort_key,
                    sort_direction: SortDirection::Desc,
                    search_term: None,
                },
            )
            .await?;
        Ok(page.items.into_iter().map(|thread| thread.id).collect())
    }

    pub async fn upsert_thread(&self, metadata: &ThreadMetadata) -> anyhow::Result<()> {
        self.threads
            .lock()
            .map_err(|_| lock_err())?
            .insert(metadata.id.to_string(), metadata.clone());
        Ok(())
    }

    pub async fn insert_thread_if_absent(&self, metadata: &ThreadMetadata) -> anyhow::Result<bool> {
        let mut threads = self.threads.lock().map_err(|_| lock_err())?;
        let key = metadata.id.to_string();
        if threads.contains_key(&key) {
            return Ok(false);
        }
        threads.insert(key, metadata.clone());
        Ok(true)
    }

    pub async fn set_thread_memory_mode(
        &self,
        thread_id: ThreadId,
        memory_mode: &str,
    ) -> anyhow::Result<bool> {
        self.thread_memory_modes
            .lock()
            .map_err(|_| lock_err())?
            .insert(thread_id.to_string(), memory_mode.to_string());
        Ok(true)
    }

    pub async fn update_thread_title(
        &self,
        thread_id: ThreadId,
        title: &str,
    ) -> anyhow::Result<bool> {
        let mut threads = self.threads.lock().map_err(|_| lock_err())?;
        let Some(thread) = threads.get_mut(&thread_id.to_string()) else {
            return Ok(false);
        };
        thread.title = title.to_string();
        Ok(true)
    }

    pub async fn touch_thread_updated_at(
        &self,
        thread_id: ThreadId,
        updated_at: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        let mut threads = self.threads.lock().map_err(|_| lock_err())?;
        let Some(thread) = threads.get_mut(&thread_id.to_string()) else {
            return Ok(false);
        };
        thread.updated_at = updated_at;
        Ok(true)
    }

    pub async fn update_thread_git_info(
        &self,
        thread_id: ThreadId,
        git_sha: Option<Option<&str>>,
        git_branch: Option<Option<&str>>,
        git_origin_url: Option<Option<&str>>,
    ) -> anyhow::Result<bool> {
        let mut threads = self.threads.lock().map_err(|_| lock_err())?;
        let Some(thread) = threads.get_mut(&thread_id.to_string()) else {
            return Ok(false);
        };
        if let Some(git_sha) = git_sha {
            thread.git_sha = git_sha.map(ToString::to_string);
        }
        if let Some(git_branch) = git_branch {
            thread.git_branch = git_branch.map(ToString::to_string);
        }
        if let Some(git_origin_url) = git_origin_url {
            thread.git_origin_url = git_origin_url.map(ToString::to_string);
        }
        Ok(true)
    }

    pub async fn apply_rollout_items(
        &self,
        builder: &ThreadMetadataBuilder,
        items: &[RolloutItem],
        new_thread_memory_mode: Option<&str>,
        updated_at_override: Option<DateTime<Utc>>,
    ) -> anyhow::Result<()> {
        if items.is_empty() {
            return Ok(());
        }
        let existing = self.get_thread(builder.id).await?;
        let mut metadata = existing
            .clone()
            .unwrap_or_else(|| builder.build(&self.default_provider));
        metadata.rollout_path = builder.rollout_path.clone();
        for item in items {
            apply_rollout_item(&mut metadata, item, &self.default_provider);
        }
        if let Some(existing) = existing.as_ref() {
            metadata.prefer_existing_git_info(existing);
        }
        if let Some(updated_at) = updated_at_override {
            metadata.updated_at = updated_at;
        }
        self.upsert_thread(&metadata).await?;
        if let Some(memory_mode) = new_thread_memory_mode {
            self.set_thread_memory_mode(builder.id, memory_mode).await?;
        }
        Ok(())
    }

    pub async fn mark_archived(
        &self,
        thread_id: ThreadId,
        rollout_path: &Path,
        archived_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        if let Some(mut metadata) = self.get_thread(thread_id).await? {
            metadata.archived_at = Some(archived_at);
            metadata.rollout_path = rollout_path.to_path_buf();
            self.upsert_thread(&metadata).await?;
        }
        Ok(())
    }

    pub async fn mark_unarchived(
        &self,
        thread_id: ThreadId,
        rollout_path: &Path,
    ) -> anyhow::Result<()> {
        if let Some(mut metadata) = self.get_thread(thread_id).await? {
            metadata.archived_at = None;
            metadata.rollout_path = rollout_path.to_path_buf();
            self.upsert_thread(&metadata).await?;
        }
        Ok(())
    }

    pub async fn delete_thread(&self, thread_id: ThreadId) -> anyhow::Result<u64> {
        let removed = self
            .threads
            .lock()
            .map_err(|_| lock_err())?
            .remove(&thread_id.to_string())
            .is_some();
        self.thread_goals.delete_thread_goal(thread_id).await?;
        Ok(u64::from(removed))
    }

    fn list_thread_spawn_children_matching(
        &self,
        parent_thread_id: ThreadId,
        status: Option<DirectionalThreadSpawnEdgeStatus>,
    ) -> anyhow::Result<Vec<ThreadId>> {
        self.spawn_edges
            .lock()
            .map_err(|_| lock_err())?
            .iter()
            .filter(|(_, (parent, edge_status))| {
                parent == &parent_thread_id.to_string()
                    && status.is_none_or(|status| status == *edge_status)
            })
            .map(|(child, _)| ThreadId::try_from(child.clone()).map_err(anyhow::Error::from))
            .collect()
    }

    fn list_thread_spawn_descendants_matching(
        &self,
        root_thread_id: ThreadId,
        status: Option<DirectionalThreadSpawnEdgeStatus>,
    ) -> anyhow::Result<Vec<ThreadId>> {
        let mut result = Vec::new();
        let mut frontier = vec![root_thread_id];
        while let Some(parent) = frontier.pop() {
            for child in self.list_thread_spawn_children_matching(parent, status)? {
                frontier.push(child);
                result.push(child);
            }
        }
        Ok(result)
    }

    fn find_thread_by_path(
        &self,
        ids: Vec<ThreadId>,
        agent_path: &str,
    ) -> anyhow::Result<Option<ThreadId>> {
        let threads = self.threads.lock().map_err(|_| lock_err())?;
        Ok(ids.into_iter().find(|id| {
            threads
                .get(&id.to_string())
                .and_then(|thread| thread.agent_path.as_deref())
                == Some(agent_path)
        }))
    }

    pub async fn insert_log(&self, _entry: &LogEntry) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn insert_logs(&self, _entries: &[LogEntry]) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn query_logs(&self, _query: &LogQuery) -> anyhow::Result<Vec<LogRow>> {
        Ok(Vec::new())
    }

    pub async fn query_feedback_logs_for_threads(
        &self,
        _thread_ids: &[String],
    ) -> anyhow::Result<Vec<u8>> {
        Ok(Vec::new())
    }

    pub async fn query_feedback_logs(&self, _thread_id: &str) -> anyhow::Result<Vec<u8>> {
        Ok(Vec::new())
    }

    pub async fn max_log_id(&self, _query: &LogQuery) -> anyhow::Result<i64> {
        Ok(0)
    }

    pub async fn get_remote_control_enrollment(
        &self,
        websocket_url: &str,
        account_id: &str,
        app_server_client_name: Option<&str>,
    ) -> anyhow::Result<Option<RemoteControlEnrollmentRecord>> {
        Ok(self
            .remote_control_enrollments
            .lock()
            .map_err(|_| lock_err())?
            .get(&remote_control_key(
                websocket_url,
                account_id,
                app_server_client_name,
            ))
            .cloned())
    }

    pub async fn upsert_remote_control_enrollment(
        &self,
        enrollment: &RemoteControlEnrollmentRecord,
    ) -> anyhow::Result<()> {
        self.remote_control_enrollments
            .lock()
            .map_err(|_| lock_err())?
            .insert(
                remote_control_key(
                    enrollment.websocket_url.as_str(),
                    enrollment.account_id.as_str(),
                    enrollment.app_server_client_name.as_deref(),
                ),
                enrollment.clone(),
            );
        Ok(())
    }

    pub async fn delete_remote_control_enrollment(
        &self,
        websocket_url: &str,
        account_id: &str,
        app_server_client_name: Option<&str>,
    ) -> anyhow::Result<u64> {
        let removed = self
            .remote_control_enrollments
            .lock()
            .map_err(|_| lock_err())?
            .remove(&remote_control_key(
                websocket_url,
                account_id,
                app_server_client_name,
            ))
            .is_some();
        Ok(u64::from(removed))
    }

    pub async fn create_agent_job(
        &self,
        _params: &AgentJobCreateParams,
        _items: &[AgentJobItemCreateParams],
    ) -> anyhow::Result<AgentJob> {
        unsupported("create_agent_job")
    }

    pub async fn get_agent_job(&self, _job_id: &str) -> anyhow::Result<Option<AgentJob>> {
        Ok(None)
    }

    pub async fn list_agent_job_items(
        &self,
        _job_id: &str,
        _status: Option<AgentJobItemStatus>,
        _limit: Option<usize>,
    ) -> anyhow::Result<Vec<AgentJobItem>> {
        Ok(Vec::new())
    }

    pub async fn get_agent_job_item(
        &self,
        _job_id: &str,
        _item_id: &str,
    ) -> anyhow::Result<Option<AgentJobItem>> {
        Ok(None)
    }

    pub async fn mark_agent_job_running(&self, _job_id: &str) -> anyhow::Result<()> {
        unsupported("mark_agent_job_running")
    }

    pub async fn mark_agent_job_completed(&self, _job_id: &str) -> anyhow::Result<()> {
        unsupported("mark_agent_job_completed")
    }

    pub async fn mark_agent_job_failed(
        &self,
        _job_id: &str,
        _error_message: &str,
    ) -> anyhow::Result<()> {
        unsupported("mark_agent_job_failed")
    }

    pub async fn mark_agent_job_cancelled(
        &self,
        _job_id: &str,
        _reason: &str,
    ) -> anyhow::Result<bool> {
        unsupported("mark_agent_job_cancelled")
    }

    pub async fn is_agent_job_cancelled(&self, _job_id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    pub async fn mark_agent_job_item_running(
        &self,
        _job_id: &str,
        _item_id: &str,
    ) -> anyhow::Result<bool> {
        unsupported("mark_agent_job_item_running")
    }

    pub async fn mark_agent_job_item_running_with_thread(
        &self,
        _job_id: &str,
        _item_id: &str,
        _thread_id: &str,
    ) -> anyhow::Result<bool> {
        unsupported("mark_agent_job_item_running_with_thread")
    }

    pub async fn mark_agent_job_item_pending(
        &self,
        _job_id: &str,
        _item_id: &str,
        _error_message: Option<&str>,
    ) -> anyhow::Result<bool> {
        unsupported("mark_agent_job_item_pending")
    }

    pub async fn set_agent_job_item_thread(
        &self,
        _job_id: &str,
        _item_id: &str,
        _thread_id: &str,
    ) -> anyhow::Result<bool> {
        unsupported("set_agent_job_item_thread")
    }

    pub async fn report_agent_job_item_result(
        &self,
        _job_id: &str,
        _item_id: &str,
        _reporting_thread_id: &str,
        _result_json: &Value,
    ) -> anyhow::Result<bool> {
        unsupported("report_agent_job_item_result")
    }

    pub async fn mark_agent_job_item_completed(
        &self,
        _job_id: &str,
        _item_id: &str,
    ) -> anyhow::Result<bool> {
        unsupported("mark_agent_job_item_completed")
    }

    pub async fn mark_agent_job_item_failed(
        &self,
        _job_id: &str,
        _item_id: &str,
        _error_message: &str,
    ) -> anyhow::Result<bool> {
        unsupported("mark_agent_job_item_failed")
    }

    pub async fn get_agent_job_progress(&self, _job_id: &str) -> anyhow::Result<AgentJobProgress> {
        Ok(AgentJobProgress {
            total_items: 0,
            pending_items: 0,
            running_items: 0,
            completed_items: 0,
            failed_items: 0,
        })
    }
}

fn remote_control_key(
    websocket_url: &str,
    account_id: &str,
    app_server_client_name: Option<&str>,
) -> String {
    format!(
        "{websocket_url}\n{account_id}\n{}",
        app_server_client_name.unwrap_or_default()
    )
}

#[derive(Clone, Default)]
pub struct GoalStore {
    goals: Arc<Mutex<HashMap<String, ThreadGoal>>>,
}

pub struct GoalUpdate {
    pub objective: Option<String>,
    pub status: Option<ThreadGoalStatus>,
    pub token_budget: Option<Option<i64>>,
    pub expected_goal_id: Option<String>,
}

pub enum GoalAccountingOutcome {
    Unchanged(Option<ThreadGoal>),
    Updated(ThreadGoal),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoalAccountingMode {
    ActiveStatusOnly,
    ActiveOnly,
    ActiveOrComplete,
    ActiveOrStopped,
}

impl GoalStore {
    pub async fn get_thread_goal(&self, thread_id: ThreadId) -> anyhow::Result<Option<ThreadGoal>> {
        Ok(self
            .goals
            .lock()
            .map_err(|_| lock_err())?
            .get(&thread_id.to_string())
            .cloned())
    }

    pub async fn replace_thread_goal(
        &self,
        thread_id: ThreadId,
        objective: &str,
        status: ThreadGoalStatus,
        token_budget: Option<i64>,
    ) -> anyhow::Result<ThreadGoal> {
        let goal = new_goal(thread_id, objective, status, token_budget);
        self.goals
            .lock()
            .map_err(|_| lock_err())?
            .insert(thread_id.to_string(), goal.clone());
        Ok(goal)
    }

    pub async fn insert_thread_goal(
        &self,
        thread_id: ThreadId,
        objective: &str,
        status: ThreadGoalStatus,
        token_budget: Option<i64>,
    ) -> anyhow::Result<Option<ThreadGoal>> {
        let mut goals = self.goals.lock().map_err(|_| lock_err())?;
        let key = thread_id.to_string();
        if goals.contains_key(&key) {
            return Ok(None);
        }
        let goal = new_goal(thread_id, objective, status, token_budget);
        goals.insert(key, goal.clone());
        Ok(Some(goal))
    }

    pub async fn update_thread_goal(
        &self,
        thread_id: ThreadId,
        update: GoalUpdate,
    ) -> anyhow::Result<Option<ThreadGoal>> {
        let mut goals = self.goals.lock().map_err(|_| lock_err())?;
        let Some(goal) = goals.get_mut(&thread_id.to_string()) else {
            return Ok(None);
        };
        if update
            .expected_goal_id
            .as_ref()
            .is_some_and(|expected| expected != &goal.goal_id)
        {
            return Ok(None);
        }
        if let Some(objective) = update.objective {
            goal.objective = objective;
        }
        if let Some(status) = update.status {
            goal.status = status;
        }
        if let Some(token_budget) = update.token_budget {
            goal.token_budget = token_budget;
        }
        if goal
            .token_budget
            .is_some_and(|budget| goal.tokens_used >= budget && goal.status.is_active())
        {
            goal.status = ThreadGoalStatus::BudgetLimited;
        }
        goal.updated_at = Utc::now();
        Ok(Some(goal.clone()))
    }

    pub async fn pause_active_thread_goal(
        &self,
        thread_id: ThreadId,
    ) -> anyhow::Result<Option<ThreadGoal>> {
        self.update_thread_goal(
            thread_id,
            GoalUpdate {
                objective: None,
                status: Some(ThreadGoalStatus::Paused),
                token_budget: None,
                expected_goal_id: None,
            },
        )
        .await
    }

    pub async fn usage_limit_active_thread_goal(
        &self,
        thread_id: ThreadId,
    ) -> anyhow::Result<Option<ThreadGoal>> {
        self.update_thread_goal(
            thread_id,
            GoalUpdate {
                objective: None,
                status: Some(ThreadGoalStatus::UsageLimited),
                token_budget: None,
                expected_goal_id: None,
            },
        )
        .await
    }

    pub async fn delete_thread_goal(&self, thread_id: ThreadId) -> anyhow::Result<bool> {
        Ok(self
            .goals
            .lock()
            .map_err(|_| lock_err())?
            .remove(&thread_id.to_string())
            .is_some())
    }

    pub async fn account_thread_goal_usage(
        &self,
        thread_id: ThreadId,
        time_delta_seconds: i64,
        token_delta: i64,
        mode: GoalAccountingMode,
        expected_goal_id: Option<&str>,
    ) -> anyhow::Result<GoalAccountingOutcome> {
        let mut goals = self.goals.lock().map_err(|_| lock_err())?;
        let Some(goal) = goals.get_mut(&thread_id.to_string()) else {
            return Ok(GoalAccountingOutcome::Unchanged(None));
        };
        if expected_goal_id.is_some_and(|goal_id| goal.goal_id != goal_id) {
            return Ok(GoalAccountingOutcome::Unchanged(Some(goal.clone())));
        }
        let eligible = match mode {
            GoalAccountingMode::ActiveStatusOnly | GoalAccountingMode::ActiveOnly => {
                goal.status == ThreadGoalStatus::Active
            }
            GoalAccountingMode::ActiveOrComplete => {
                matches!(
                    goal.status,
                    ThreadGoalStatus::Active | ThreadGoalStatus::Complete
                )
            }
            GoalAccountingMode::ActiveOrStopped => matches!(
                goal.status,
                ThreadGoalStatus::Active
                    | ThreadGoalStatus::Paused
                    | ThreadGoalStatus::Blocked
                    | ThreadGoalStatus::UsageLimited
                    | ThreadGoalStatus::BudgetLimited
                    | ThreadGoalStatus::Complete
            ),
        };
        if !eligible {
            return Ok(GoalAccountingOutcome::Unchanged(Some(goal.clone())));
        }
        goal.tokens_used = goal.tokens_used.saturating_add(token_delta.max(0));
        goal.time_used_seconds = goal
            .time_used_seconds
            .saturating_add(time_delta_seconds.max(0));
        if goal
            .token_budget
            .is_some_and(|budget| goal.tokens_used >= budget && goal.status.is_active())
        {
            goal.status = ThreadGoalStatus::BudgetLimited;
        }
        goal.updated_at = Utc::now();
        Ok(GoalAccountingOutcome::Updated(goal.clone()))
    }
}

fn new_goal(
    thread_id: ThreadId,
    objective: &str,
    status: ThreadGoalStatus,
    token_budget: Option<i64>,
) -> ThreadGoal {
    let now = Utc::now();
    let status = if token_budget == Some(0) && status.is_active() {
        ThreadGoalStatus::BudgetLimited
    } else {
        status
    };
    ThreadGoal {
        thread_id,
        goal_id: Uuid::new_v4().to_string(),
        objective: objective.to_string(),
        status,
        token_budget,
        tokens_used: 0,
        time_used_seconds: 0,
        created_at: now,
        updated_at: now,
    }
}

#[derive(Clone, Copy, Default)]
pub struct MemoryStore;

impl MemoryStore {
    pub async fn clear_memory_data(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn record_stage1_output_usage(
        &self,
        _thread_ids: &[ThreadId],
    ) -> anyhow::Result<usize> {
        Ok(0)
    }

    pub async fn claim_stage1_jobs_for_startup(
        &self,
        _current_thread_id: ThreadId,
        _params: Stage1StartupClaimParams<'_>,
    ) -> anyhow::Result<Vec<Stage1JobClaim>> {
        Ok(Vec::new())
    }

    pub async fn list_stage1_outputs_for_global(
        &self,
        _n: usize,
    ) -> anyhow::Result<Vec<Stage1Output>> {
        Ok(Vec::new())
    }

    pub async fn prune_stage1_outputs_for_retention(
        &self,
        _max_unused_days: i64,
        _limit: usize,
    ) -> anyhow::Result<usize> {
        Ok(0)
    }

    pub async fn get_phase2_input_selection(
        &self,
        _n: usize,
        _max_unused_days: i64,
    ) -> anyhow::Result<Vec<Stage1Output>> {
        Ok(Vec::new())
    }

    pub async fn mark_thread_memory_mode_polluted(
        &self,
        _thread_id: ThreadId,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    pub async fn try_claim_stage1_job(
        &self,
        _thread_id: ThreadId,
        _worker_id: ThreadId,
        _source_updated_at: i64,
        _lease_seconds: i64,
        _max_running_jobs: usize,
    ) -> anyhow::Result<Stage1JobClaimOutcome> {
        Ok(Stage1JobClaimOutcome::SkippedRunning)
    }

    pub async fn mark_stage1_job_succeeded(
        &self,
        _thread_id: ThreadId,
        _ownership_token: &str,
        _source_updated_at: i64,
        _raw_memory: &str,
        _rollout_summary: &str,
        _rollout_slug: Option<&str>,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    pub async fn mark_stage1_job_succeeded_no_output(
        &self,
        _thread_id: ThreadId,
        _ownership_token: &str,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    pub async fn mark_stage1_job_failed(
        &self,
        _thread_id: ThreadId,
        _ownership_token: &str,
        _failure_reason: &str,
        _retry_delay_seconds: i64,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    pub async fn enqueue_global_consolidation(&self, _input_watermark: i64) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn try_claim_global_phase2_job(
        &self,
        _worker_id: ThreadId,
        _lease_seconds: i64,
    ) -> anyhow::Result<Phase2JobClaimOutcome> {
        Ok(Phase2JobClaimOutcome::SkippedRunning)
    }

    pub async fn heartbeat_global_phase2_job(
        &self,
        _ownership_token: &str,
        _lease_seconds: i64,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    pub async fn mark_global_phase2_job_succeeded(
        &self,
        _ownership_token: &str,
        _completed_watermark: i64,
        _selected_outputs: &[Stage1Output],
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    pub async fn mark_global_phase2_job_failed(
        &self,
        _ownership_token: &str,
        _failure_reason: &str,
        _retry_delay_seconds: i64,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    pub async fn mark_global_phase2_job_failed_if_unowned(
        &self,
        _ownership_token: &str,
        _failure_reason: &str,
        _retry_delay_seconds: i64,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteControlEnrollmentRecord {
    pub websocket_url: String,
    pub account_id: String,
    pub app_server_client_name: Option<String>,
    pub server_id: String,
    pub environment_id: String,
    pub server_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDbPath {
    pub label: &'static str,
    pub path: PathBuf,
}

#[derive(Clone, Copy)]
pub struct ThreadFilterOptions<'a> {
    pub archived_only: bool,
    pub allowed_sources: &'a [String],
    pub model_providers: Option<&'a [String]>,
    pub cwd_filters: Option<&'a [PathBuf]>,
    pub anchor: Option<&'a Anchor>,
    pub sort_key: SortKey,
    pub sort_direction: SortDirection,
    pub search_term: Option<&'a str>,
}

pub fn state_db_filename() -> String {
    crate::STATE_DB_FILENAME.to_string()
}

pub fn state_db_path(codex_home: &Path) -> PathBuf {
    codex_home.join(crate::STATE_DB_FILENAME)
}

pub fn logs_db_filename() -> String {
    crate::LOGS_DB_FILENAME.to_string()
}

pub fn logs_db_path(codex_home: &Path) -> PathBuf {
    codex_home.join(crate::LOGS_DB_FILENAME)
}

pub fn goals_db_filename() -> String {
    crate::GOALS_DB_FILENAME.to_string()
}

pub fn goals_db_path(codex_home: &Path) -> PathBuf {
    codex_home.join(crate::GOALS_DB_FILENAME)
}

pub fn memories_db_filename() -> String {
    crate::MEMORIES_DB_FILENAME.to_string()
}

pub fn memories_db_path(codex_home: &Path) -> PathBuf {
    codex_home.join(crate::MEMORIES_DB_FILENAME)
}

pub fn runtime_db_paths(codex_home: &Path) -> Vec<RuntimeDbPath> {
    vec![
        RuntimeDbPath {
            label: "state DB",
            path: state_db_path(codex_home),
        },
        RuntimeDbPath {
            label: "log DB",
            path: logs_db_path(codex_home),
        },
        RuntimeDbPath {
            label: "goals DB",
            path: goals_db_path(codex_home),
        },
        RuntimeDbPath {
            label: "memories DB",
            path: memories_db_path(codex_home),
        },
    ]
}

pub async fn sqlite_integrity_check(_path: &Path) -> anyhow::Result<Vec<String>> {
    unsupported("sqlite_integrity_check")
}

#[derive(Debug, Clone)]
pub struct ThreadStateAuditRow {
    pub id: String,
    pub rollout_path: PathBuf,
    pub archived: bool,
    pub source: String,
    pub model_provider: String,
}

pub async fn read_thread_state_audit_rows(_path: &Path) -> Result<Vec<ThreadStateAuditRow>> {
    unsupported("read_thread_state_audit_rows")
}

pub trait DbTelemetry: Send + Sync + 'static {
    fn counter(&self, name: &str, inc: i64, tags: &[(&str, &str)]);
    fn record_duration(&self, name: &str, duration: Duration, tags: &[(&str, &str)]);
}

pub type DbTelemetryHandle = Arc<dyn DbTelemetry>;

static PROCESS_DB_TELEMETRY: OnceLock<DbTelemetryHandle> = OnceLock::new();

pub fn install_process_db_telemetry(telemetry: DbTelemetryHandle) -> bool {
    PROCESS_DB_TELEMETRY.set(telemetry).is_ok()
}

pub fn record_backfill_gate(
    telemetry: Option<&dyn DbTelemetry>,
    duration: Duration,
    result: &anyhow::Result<()>,
) {
    let status = if result.is_ok() { "success" } else { "failed" };
    if let Some(telemetry) = telemetry.or_else(|| PROCESS_DB_TELEMETRY.get().map(AsRef::as_ref)) {
        telemetry.counter(crate::DB_INIT_METRIC, 1, &[("status", status)]);
        telemetry.record_duration(
            crate::DB_INIT_DURATION_METRIC,
            duration,
            &[("status", status)],
        );
    }
}

pub fn record_fallback(
    caller: &'static str,
    reason: &'static str,
    telemetry_override: Option<&dyn DbTelemetry>,
) {
    if let Some(telemetry) =
        telemetry_override.or_else(|| PROCESS_DB_TELEMETRY.get().map(AsRef::as_ref))
    {
        telemetry.counter(
            crate::DB_FALLBACK_METRIC,
            1,
            &[("caller", caller), ("reason", reason)],
        );
    }
}
