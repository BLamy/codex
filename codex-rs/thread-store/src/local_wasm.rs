use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use codex_protocol::ThreadId;
use codex_rollout::StateDbHandle;
use tokio::sync::Mutex;

use crate::AppendThreadItemsParams;
use crate::ArchiveThreadParams;
use crate::CreateThreadParams;
use crate::DeleteThreadParams;
use crate::InMemoryThreadStore;
use crate::ListThreadsParams;
use crate::LoadThreadHistoryParams;
use crate::ReadThreadByRolloutPathParams;
use crate::ReadThreadParams;
use crate::ResumeThreadParams;
use crate::StoredModelContext;
use crate::StoredThread;
use crate::StoredThreadHistory;
use crate::ThreadPage;
use crate::ThreadStore;
use crate::ThreadStoreError;
use crate::ThreadStoreFuture;
use crate::ThreadStoreResult;
use crate::UpdateThreadMetadataParams;

/// Browser implementation of the local thread store.
///
/// The native implementation persists rollout JSONL and SQLite indexes. Browser
/// Codex keeps the same public store identity and lifecycle contract, but backs
/// it with the process-scoped in-memory store until the host exposes durable
/// virtual-filesystem storage for rollouts.
#[derive(Clone)]
pub struct LocalThreadStore {
    config: LocalThreadStoreConfig,
    inner: Arc<InMemoryThreadStore>,
    state_db: Option<StateDbHandle>,
    rollout_paths: Arc<Mutex<HashMap<ThreadId, PathBuf>>>,
}

/// Process-scoped configuration for browser-local thread storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalThreadStoreConfig {
    pub codex_home: PathBuf,
    pub sqlite_home: PathBuf,
    pub default_model_provider_id: String,
}

impl LocalThreadStoreConfig {
    pub fn from_config(config: &impl codex_rollout::RolloutConfigView) -> Self {
        Self {
            codex_home: config.codex_home().to_path_buf(),
            sqlite_home: config.sqlite_home().to_path_buf(),
            default_model_provider_id: config.model_provider_id().to_string(),
        }
    }
}

impl std::fmt::Debug for LocalThreadStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalThreadStore")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl LocalThreadStore {
    pub fn new(config: LocalThreadStoreConfig, state_db: Option<StateDbHandle>) -> Self {
        let store_id = format!("browser-local:{}", config.codex_home.display());
        Self {
            config,
            inner: InMemoryThreadStore::for_id(store_id),
            state_db,
            rollout_paths: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn state_db(&self) -> Option<StateDbHandle> {
        self.state_db.clone()
    }

    pub async fn read_thread_by_rollout_path(
        &self,
        rollout_path: PathBuf,
        include_archived: bool,
        include_history: bool,
    ) -> ThreadStoreResult<StoredThread> {
        ThreadStore::read_thread_by_rollout_path(
            self,
            ReadThreadByRolloutPathParams {
                rollout_path,
                include_archived,
                include_history,
            },
        )
        .await
    }

    pub async fn live_rollout_path(&self, thread_id: ThreadId) -> ThreadStoreResult<PathBuf> {
        let paths = self.rollout_paths.lock().await;
        paths
            .get(&thread_id)
            .cloned()
            .ok_or(ThreadStoreError::ThreadNotFound { thread_id })
    }

    fn synthetic_rollout_path(&self, thread_id: ThreadId) -> PathBuf {
        self.config
            .codex_home
            .join("sessions")
            .join(format!("{thread_id}.jsonl"))
    }
}

impl ThreadStore for LocalThreadStore {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn create_thread(&self, params: CreateThreadParams) -> ThreadStoreFuture<'_, ()> {
        Box::pin(async move {
            let thread_id = params.thread_id;
            let metadata = params.metadata.clone();
            let rollout_path = self.synthetic_rollout_path(thread_id);
            self.inner.create_thread(params).await?;
            self.inner
                .resume_thread(ResumeThreadParams {
                    thread_id,
                    rollout_path: Some(rollout_path.clone()),
                    history: None,
                    include_archived: true,
                    metadata,
                })
                .await?;
            self.rollout_paths
                .lock()
                .await
                .insert(thread_id, rollout_path);
            Ok(())
        })
    }

    fn resume_thread(&self, mut params: ResumeThreadParams) -> ThreadStoreFuture<'_, ()> {
        Box::pin(async move {
            let thread_id = params.thread_id;
            let rollout_path = params
                .rollout_path
                .clone()
                .unwrap_or_else(|| self.synthetic_rollout_path(thread_id));
            params.rollout_path = Some(rollout_path.clone());
            self.inner.resume_thread(params).await?;
            self.rollout_paths
                .lock()
                .await
                .insert(thread_id, rollout_path);
            Ok(())
        })
    }

    fn append_items(&self, params: AppendThreadItemsParams) -> ThreadStoreFuture<'_, ()> {
        self.inner.append_items(params)
    }

    fn persist_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        self.inner.persist_thread(thread_id)
    }

    fn flush_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        self.inner.flush_thread(thread_id)
    }

    fn shutdown_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        self.inner.shutdown_thread(thread_id)
    }

    fn discard_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        self.inner.discard_thread(thread_id)
    }

    fn load_history(
        &self,
        params: LoadThreadHistoryParams,
    ) -> ThreadStoreFuture<'_, StoredThreadHistory> {
        self.inner.load_history(params)
    }

    fn load_latest_model_context(
        &self,
        params: LoadThreadHistoryParams,
    ) -> ThreadStoreFuture<'_, StoredModelContext> {
        self.inner.load_latest_model_context(params)
    }

    fn read_thread(&self, params: ReadThreadParams) -> ThreadStoreFuture<'_, StoredThread> {
        self.inner.read_thread(params)
    }

    fn read_thread_by_rollout_path(
        &self,
        params: ReadThreadByRolloutPathParams,
    ) -> ThreadStoreFuture<'_, StoredThread> {
        self.inner.read_thread_by_rollout_path(params)
    }

    fn list_threads(&self, params: ListThreadsParams) -> ThreadStoreFuture<'_, ThreadPage> {
        self.inner.list_threads(params)
    }

    fn update_thread_metadata(
        &self,
        params: UpdateThreadMetadataParams,
    ) -> ThreadStoreFuture<'_, StoredThread> {
        self.inner.update_thread_metadata(params)
    }

    fn archive_thread(&self, params: ArchiveThreadParams) -> ThreadStoreFuture<'_, ()> {
        self.inner.archive_thread(params)
    }

    fn unarchive_thread(&self, params: ArchiveThreadParams) -> ThreadStoreFuture<'_, StoredThread> {
        self.inner.unarchive_thread(params)
    }

    fn delete_thread(&self, params: DeleteThreadParams) -> ThreadStoreFuture<'_, ()> {
        Box::pin(async move {
            let thread_id = params.thread_id;
            self.inner.delete_thread(params).await?;
            self.rollout_paths.lock().await.remove(&thread_id);
            Ok(())
        })
    }
}
