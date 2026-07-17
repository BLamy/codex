//! SQLite-backed state for rollout metadata.
//!
//! This crate is intentionally small and focused: it extracts rollout metadata
//! from JSONL rollouts and mirrors it into a local SQLite database. Backfill
//! orchestration and rollout scanning live in `codex-core`.

#[cfg(not(target_arch = "wasm32"))]
const _: () = assert!(
    libsqlite3_sys::SQLITE_VERSION_NUMBER >= 3_051_003,
    "bundled SQLite must include the WAL-reset corruption fix",
);

#[cfg(not(target_arch = "wasm32"))]
mod audit;
#[cfg(not(target_arch = "wasm32"))]
mod extract;
#[cfg(not(target_arch = "wasm32"))]
pub mod log_db;
#[cfg(target_arch = "wasm32")]
#[path = "log_db_wasm.rs"]
pub mod log_db;
#[cfg(not(target_arch = "wasm32"))]
mod migrations;
#[cfg(not(target_arch = "wasm32"))]
mod model;
#[cfg(not(target_arch = "wasm32"))]
mod paths;
#[cfg(not(target_arch = "wasm32"))]
mod runtime;
#[cfg(not(target_arch = "wasm32"))]
mod telemetry;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use model::LogEntry;
#[cfg(not(target_arch = "wasm32"))]
pub use model::LogQuery;
#[cfg(not(target_arch = "wasm32"))]
pub use model::LogRow;
#[cfg(not(target_arch = "wasm32"))]
pub use model::Phase2JobClaimOutcome;
/// Preferred entrypoint: owns configuration and metrics.
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::StateRuntime;

#[cfg(not(target_arch = "wasm32"))]
pub use audit::ThreadStateAuditRow;
#[cfg(not(target_arch = "wasm32"))]
pub use audit::read_thread_state_audit_rows;
/// Low-level storage engine: useful for focused tests.
///
/// Most consumers should prefer [`StateRuntime`].
#[cfg(not(target_arch = "wasm32"))]
pub use extract::apply_rollout_item;
#[cfg(not(target_arch = "wasm32"))]
pub use extract::rollout_item_affects_thread_metadata;
#[cfg(not(target_arch = "wasm32"))]
pub use model::AgentJob;
#[cfg(not(target_arch = "wasm32"))]
pub use model::AgentJobCreateParams;
#[cfg(not(target_arch = "wasm32"))]
pub use model::AgentJobItem;
#[cfg(not(target_arch = "wasm32"))]
pub use model::AgentJobItemCreateParams;
#[cfg(not(target_arch = "wasm32"))]
pub use model::AgentJobItemStatus;
#[cfg(not(target_arch = "wasm32"))]
pub use model::AgentJobProgress;
#[cfg(not(target_arch = "wasm32"))]
pub use model::AgentJobStatus;
#[cfg(not(target_arch = "wasm32"))]
pub use model::Anchor;
#[cfg(not(target_arch = "wasm32"))]
pub use model::BackfillState;
#[cfg(not(target_arch = "wasm32"))]
pub use model::BackfillStats;
#[cfg(not(target_arch = "wasm32"))]
pub use model::BackfillStatus;
#[cfg(not(target_arch = "wasm32"))]
pub use model::DirectionalThreadSpawnEdgeStatus;
#[cfg(not(target_arch = "wasm32"))]
pub use model::ExtractionOutcome;
#[cfg(not(target_arch = "wasm32"))]
pub use model::SortDirection;
#[cfg(not(target_arch = "wasm32"))]
pub use model::SortKey;
#[cfg(not(target_arch = "wasm32"))]
pub use model::Stage1JobClaim;
#[cfg(not(target_arch = "wasm32"))]
pub use model::Stage1JobClaimOutcome;
#[cfg(not(target_arch = "wasm32"))]
pub use model::Stage1Output;
#[cfg(not(target_arch = "wasm32"))]
pub use model::Stage1StartupClaimParams;
#[cfg(not(target_arch = "wasm32"))]
pub use model::ThreadGoal;
#[cfg(not(target_arch = "wasm32"))]
pub use model::ThreadGoalStatus;
#[cfg(not(target_arch = "wasm32"))]
pub use model::ThreadMetadata;
#[cfg(not(target_arch = "wasm32"))]
pub use model::ThreadMetadataBuilder;
#[cfg(not(target_arch = "wasm32"))]
pub use model::ThreadRelationFilter;
#[cfg(not(target_arch = "wasm32"))]
pub use model::ThreadsPage;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::ExternalAgentConfigImportDetailsRecord;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::ExternalAgentConfigImportFailureRecord;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::ExternalAgentConfigImportHistoryRecord;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::ExternalAgentConfigImportSuccessRecord;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::GoalAccountingMode;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::GoalAccountingOutcome;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::GoalStore;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::GoalUpdate;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::MemoryStore;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::RemoteControlEnrollmentRecord;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::RuntimeDbBackup;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::RuntimeDbPath;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::ThreadFilterOptions;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::backup_runtime_db_for_fresh_start;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::goals_db_filename;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::goals_db_path;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::is_sqlite_corruption_error;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::logs_db_filename;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::logs_db_path;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::memories_db_filename;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::memories_db_path;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::runtime_db_path_for_corruption_error;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::runtime_db_paths;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::sqlite_error_detail_is_corruption;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::sqlite_error_detail_is_lock;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::sqlite_integrity_check;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::state_db_filename;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::state_db_path;
#[cfg(not(target_arch = "wasm32"))]
pub use telemetry::DbTelemetry;
#[cfg(not(target_arch = "wasm32"))]
pub use telemetry::DbTelemetryHandle;
#[cfg(not(target_arch = "wasm32"))]
pub use telemetry::install_process_db_telemetry;
#[cfg(not(target_arch = "wasm32"))]
pub use telemetry::record_backfill_gate;
#[cfg(not(target_arch = "wasm32"))]
pub use telemetry::record_fallback;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

/// Environment variable for overriding the SQLite state database home directory.
pub const SQLITE_HOME_ENV: &str = "CODEX_SQLITE_HOME";

pub const LOGS_DB_FILENAME: &str = "logs_2.sqlite";
pub const GOALS_DB_FILENAME: &str = "goals_1.sqlite";
pub const MEMORIES_DB_FILENAME: &str = "memories_1.sqlite";
pub const STATE_DB_FILENAME: &str = "state_5.sqlite";

/// Errors encountered during DB operations. Tags: [stage]
pub const DB_ERROR_METRIC: &str = "codex.db.error";
/// Metrics on backfill process. Tags: [status]
pub const DB_METRIC_BACKFILL: &str = "codex.db.backfill";
/// Metrics on backfill duration. Tags: [status]
pub const DB_METRIC_BACKFILL_DURATION_MS: &str = "codex.db.backfill.duration_ms";
/// SQLite initialization attempts. Tags: [status, phase, db, error]
pub const DB_INIT_METRIC: &str = "codex.sqlite.init.count";
/// SQLite initialization latency. Tags: [status, phase, db, error]
pub const DB_INIT_DURATION_METRIC: &str = "codex.sqlite.init.duration_ms";
/// Rollout fallback attempts. Tags: [caller, reason]
pub const DB_FALLBACK_METRIC: &str = "codex.sqlite.fallback.count";
