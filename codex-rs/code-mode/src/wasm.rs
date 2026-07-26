//! Browser-safe code-mode facade.
//!
//! Upstream code mode embeds V8 or spawns a native helper process. Neither is
//! available on `wasm32-unknown-unknown`, so this facade preserves the public
//! session API and returns an explicit unsupported result without compiling V8.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use codex_code_mode_protocol::CellId;
use codex_code_mode_protocol::CodeModeNestedToolCall;
use codex_code_mode_protocol::CodeModeSession;
use codex_code_mode_protocol::CodeModeSessionDelegate;
use codex_code_mode_protocol::CodeModeSessionProvider;
use codex_code_mode_protocol::CodeModeSessionProviderFuture;
use codex_code_mode_protocol::CodeModeSessionResultFuture;
use codex_code_mode_protocol::ExecuteRequest;
use codex_code_mode_protocol::ExecuteToPendingOutcome;
use codex_code_mode_protocol::FunctionCallOutputContentItem;
use codex_code_mode_protocol::NotificationFuture;
use codex_code_mode_protocol::RuntimeResponse;
use codex_code_mode_protocol::StartedCell;
use codex_code_mode_protocol::ToolInvocationFuture;
use codex_code_mode_protocol::WaitOutcome;
use codex_code_mode_protocol::WaitRequest;
use codex_code_mode_protocol::WaitToPendingOutcome;
use codex_code_mode_protocol::WaitToPendingRequest;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

const UNAVAILABLE: &str = "browser code-mode execution requires an almostnode host runtime shim";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum V8JitMode {
    Enabled,
    Disabled,
}

pub fn initialize_v8(_jit_mode: V8JitMode) -> Result<(), String> {
    Err(UNAVAILABLE.to_string())
}

pub struct NoopCodeModeSessionDelegate;

impl CodeModeSessionDelegate for NoopCodeModeSessionDelegate {
    fn invoke_tool<'a>(
        &'a self,
        _invocation: CodeModeNestedToolCall,
        cancellation_token: CancellationToken,
    ) -> ToolInvocationFuture<'a> {
        Box::pin(async move {
            cancellation_token.cancelled().await;
            Err("code mode nested tools are unavailable".to_string())
        })
    }

    fn notify<'a>(
        &'a self,
        _call_id: String,
        _cell_id: CellId,
        _text: String,
        _cancellation_token: CancellationToken,
    ) -> NotificationFuture<'a> {
        Box::pin(async { Ok(()) })
    }

    fn cell_closed(&self, _cell_id: &CellId) {}
}

#[derive(Default)]
pub struct InProcessCodeModeSessionProvider;

impl CodeModeSessionProvider for InProcessCodeModeSessionProvider {
    fn create_session<'a>(
        &'a self,
        delegate: Arc<dyn CodeModeSessionDelegate>,
    ) -> CodeModeSessionProviderFuture<'a> {
        Box::pin(async move {
            let session: Arc<dyn CodeModeSession> =
                Arc::new(InProcessCodeModeSession::with_delegate(delegate));
            Ok(session)
        })
    }
}

pub struct InProcessCodeModeSession {
    _delegate: Arc<dyn CodeModeSessionDelegate>,
    next_cell_id: AtomicU64,
}

impl InProcessCodeModeSession {
    pub fn new() -> Self {
        Self::with_delegate(Arc::new(NoopCodeModeSessionDelegate))
    }

    pub fn with_delegate(delegate: Arc<dyn CodeModeSessionDelegate>) -> Self {
        Self {
            _delegate: delegate,
            next_cell_id: AtomicU64::new(1),
        }
    }

    pub fn with_delegate_and_task_failure_handler(
        delegate: Arc<dyn CodeModeSessionDelegate>,
        _task_failure_handler: crate::TaskFailureHandler,
    ) -> Self {
        Self::with_delegate(delegate)
    }

    pub async fn execute(&self, _request: ExecuteRequest) -> Result<StartedCell, String> {
        let cell_id = self.allocate_cell_id();
        let (response_tx, response_rx) = oneshot::channel();
        let _ = response_tx.send(unavailable_response(cell_id.clone()));
        Ok(StartedCell::new(cell_id, response_rx))
    }

    pub async fn execute_to_pending(
        &self,
        _request: ExecuteRequest,
    ) -> Result<ExecuteToPendingOutcome, String> {
        Ok(ExecuteToPendingOutcome::Completed(unavailable_response(
            self.allocate_cell_id(),
        )))
    }

    pub async fn wait(&self, request: WaitRequest) -> Result<WaitOutcome, String> {
        Ok(WaitOutcome::MissingCell(missing_cell_response(
            request.cell_id,
        )))
    }

    pub async fn terminate(&self, cell_id: CellId) -> Result<WaitOutcome, String> {
        Ok(WaitOutcome::MissingCell(missing_cell_response(cell_id)))
    }

    pub async fn wait_to_pending(
        &self,
        request: WaitToPendingRequest,
    ) -> Result<WaitToPendingOutcome, String> {
        Ok(WaitToPendingOutcome::MissingCell(missing_cell_response(
            request.cell_id,
        )))
    }

    pub async fn shutdown(&self) -> Result<(), String> {
        Ok(())
    }

    fn allocate_cell_id(&self) -> CellId {
        CellId::new(
            self.next_cell_id
                .fetch_add(1, Ordering::Relaxed)
                .to_string(),
        )
    }
}

impl Default for InProcessCodeModeSession {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeModeSession for InProcessCodeModeSession {
    fn execute<'a>(
        &'a self,
        request: ExecuteRequest,
    ) -> CodeModeSessionResultFuture<'a, StartedCell> {
        Box::pin(InProcessCodeModeSession::execute(self, request))
    }

    fn wait<'a>(&'a self, request: WaitRequest) -> CodeModeSessionResultFuture<'a, WaitOutcome> {
        Box::pin(InProcessCodeModeSession::wait(self, request))
    }

    fn terminate<'a>(&'a self, cell_id: CellId) -> CodeModeSessionResultFuture<'a, WaitOutcome> {
        Box::pin(InProcessCodeModeSession::terminate(self, cell_id))
    }

    fn shutdown<'a>(&'a self) -> CodeModeSessionResultFuture<'a, ()> {
        Box::pin(InProcessCodeModeSession::shutdown(self))
    }
}

#[derive(Default)]
pub struct ProcessOwnedCodeModeSessionProvider;

impl ProcessOwnedCodeModeSessionProvider {
    pub fn with_host_program(_host_program: PathBuf) -> Self {
        Self
    }
}

impl CodeModeSessionProvider for ProcessOwnedCodeModeSessionProvider {
    fn create_session<'a>(
        &'a self,
        delegate: Arc<dyn CodeModeSessionDelegate>,
    ) -> CodeModeSessionProviderFuture<'a> {
        InProcessCodeModeSessionProvider.create_session(delegate)
    }
}

#[derive(Default)]
pub struct ProcessOwnedCodeModeSession {
    inner: InProcessCodeModeSession,
}

impl ProcessOwnedCodeModeSession {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CodeModeSession for ProcessOwnedCodeModeSession {
    fn execute<'a>(
        &'a self,
        request: ExecuteRequest,
    ) -> CodeModeSessionResultFuture<'a, StartedCell> {
        CodeModeSession::execute(&self.inner, request)
    }

    fn wait<'a>(&'a self, request: WaitRequest) -> CodeModeSessionResultFuture<'a, WaitOutcome> {
        CodeModeSession::wait(&self.inner, request)
    }

    fn terminate<'a>(&'a self, cell_id: CellId) -> CodeModeSessionResultFuture<'a, WaitOutcome> {
        CodeModeSession::terminate(&self.inner, cell_id)
    }

    fn shutdown<'a>(&'a self) -> CodeModeSessionResultFuture<'a, ()> {
        CodeModeSession::shutdown(&self.inner)
    }
}

fn unavailable_response(cell_id: CellId) -> RuntimeResponse {
    RuntimeResponse::Result {
        cell_id,
        content_items: vec![FunctionCallOutputContentItem::InputText {
            text: UNAVAILABLE.to_string(),
        }],
        error_text: Some(UNAVAILABLE.to_string()),
    }
}

fn missing_cell_response(cell_id: CellId) -> RuntimeResponse {
    RuntimeResponse::Result {
        error_text: Some(format!("exec cell {cell_id} not found")),
        cell_id,
        content_items: Vec::new(),
    }
}
