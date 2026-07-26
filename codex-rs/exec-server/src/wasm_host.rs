use std::collections::HashMap;
use std::collections::VecDeque;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::Weak;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use codex_utils_absolute_path::AbsolutePathBuf;
use futures::FutureExt;
use futures::future::LocalBoxFuture;
use js_sys::Promise;
use js_sys::Reflect;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::time::Duration;
use tokio::time::timeout;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::CopyOptions;
use crate::CreateDirectoryOptions;
use crate::ExecBackend;
use crate::ExecBackendFuture;
use crate::ExecProcess;
use crate::ExecProcessEvent;
use crate::ExecProcessEventReceiver;
use crate::ExecProcessFuture;
use crate::ExecServerError;
use crate::ExecutorFileSystem;
use crate::ExecutorFileSystemFuture;
use crate::FileMetadata;
use crate::FileSystemReadStream;
use crate::FileSystemResult;
use crate::FileSystemSandboxContext;
use crate::HttpClient;
use crate::HttpHeader;
use crate::HttpRequestParams;
use crate::HttpRequestResponse;
use crate::HttpResponseBodyStream;
use crate::ProcessId;
use crate::ProcessSignal;
use crate::ReadDirectoryEntry;
use crate::RemoveOptions;
use crate::StartedExecProcess;
use crate::WriteResponse;
use crate::WriteStatus;
use crate::process::ExecProcessEventLog;
use crate::protocol::ByteChunk;
use crate::protocol::ExecOutputStream;
use crate::protocol::ExecParams;
use crate::protocol::ProcessOutputChunk;
use crate::protocol::ReadResponse;
use bytes::Bytes;
use codex_utils_path_uri::PathUri;

const RETAINED_OUTPUT_BYTES_PER_PROCESS: usize = 1024 * 1024;
const RETAINED_OUTPUT_EVENTS_PER_PROCESS: usize = 512;

static WASM_HOST_PROCESSES: LazyLock<Mutex<HashMap<String, Weak<WasmHostExecProcess>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis, js_name = __almostnodeCodexHostRequest, catch)]
    fn almostnode_codex_host_request(op: &str, params: JsValue) -> Result<Promise, JsValue>;
}

#[derive(Clone, Debug, Default)]
pub struct WasmHostProcess;

#[derive(Clone, Debug, Default)]
pub struct WasmHostFileSystem;

#[derive(Clone, Debug, Default)]
pub struct WasmHostHttpClient;

pub static LOCAL_FS: LazyLock<Arc<dyn ExecutorFileSystem>> =
    LazyLock::new(|| -> Arc<dyn ExecutorFileSystem> { Arc::new(WasmHostFileSystem) });

#[derive(Default)]
struct WasmProcessState {
    output: VecDeque<ProcessOutputChunk>,
    retained_bytes: usize,
    next_seq: u64,
    exit_code: Option<i32>,
    closed: bool,
    failure: Option<String>,
}

struct WasmHostExecProcess {
    process_id: ProcessId,
    state: Arc<Mutex<WasmProcessState>>,
    wake_tx: watch::Sender<u64>,
    events: ExecProcessEventLog,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostProcessSpawnParams<'a> {
    process_handle: &'a str,
    command: &'a [String],
    cwd: String,
    env: &'a HashMap<String, String>,
    tty: bool,
    stream_stdin: bool,
    stream_stdout_stderr: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostProcessSpawnResult {
    process_handle: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostProcessEventEnvelope {
    event: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostProcessOutputDelta {
    process_handle: String,
    stream: String,
    delta_base64: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostProcessExited {
    process_handle: String,
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    stderr: String,
    #[serde(default)]
    exit_code: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostReadFileResult {
    content: String,
    encoding: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostReadDirectoryResult {
    entries: Vec<HostReadDirectoryEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostReadDirectoryEntry {
    #[serde(alias = "fileName")]
    name: String,
    #[serde(default)]
    r#type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostMetadataResult {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    mtime_ms: f64,
    #[serde(default)]
    size: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostFetchResult {
    status: u16,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    body_base64: String,
}

impl ExecBackend for WasmHostProcess {
    fn start(&self, params: ExecParams) -> ExecBackendFuture<'_> {
        Box::pin(async move { self.start_impl(params).await })
    }
}

impl WasmHostProcess {
    async fn start_impl(&self, params: ExecParams) -> Result<StartedExecProcess, ExecServerError> {
        let process_id = params.process_id.clone();
        let (wake_tx, _wake_rx) = watch::channel(0);
        let process = Arc::new(WasmHostExecProcess {
            process_id: process_id.clone(),
            state: Arc::new(Mutex::new(WasmProcessState {
                next_seq: 1,
                ..WasmProcessState::default()
            })),
            wake_tx,
            events: ExecProcessEventLog::new(
                RETAINED_OUTPUT_EVENTS_PER_PROCESS,
                RETAINED_OUTPUT_BYTES_PER_PROCESS,
            ),
        });

        register_wasm_host_process(&process);
        let task_process = Arc::clone(&process);
        wasm_bindgen_futures::spawn_local(async move {
            let host_params = HostProcessSpawnParams {
                process_handle: process_id.as_str(),
                command: &params.argv,
                cwd: path_uri_to_host_string(&params.cwd),
                env: &params.env,
                tty: params.tty,
                stream_stdin: params.tty || params.pipe_stdin,
                stream_stdout_stderr: true,
            };
            match host_request_json::<HostProcessSpawnResult, _>("process/spawn", &host_params)
                .await
            {
                Ok(result) => {
                    if result.process_handle != process_id.as_str() {
                        task_process.fail(format!(
                            "host process/spawn returned handle `{}` for `{}`",
                            result.process_handle,
                            process_id.as_str()
                        ));
                    }
                }
                Err(error) => {
                    task_process.fail(error.to_string());
                }
            }
        });

        Ok(StartedExecProcess { process })
    }
}

impl ExecProcess for WasmHostExecProcess {
    fn process_id(&self) -> &ProcessId {
        &self.process_id
    }

    fn subscribe_wake(&self) -> watch::Receiver<u64> {
        self.wake_tx.subscribe()
    }

    fn subscribe_events(&self) -> ExecProcessEventReceiver {
        self.events.subscribe()
    }

    fn read(
        &self,
        after_seq: Option<u64>,
        max_bytes: Option<usize>,
        wait_ms: Option<u64>,
    ) -> ExecProcessFuture<'_, ReadResponse> {
        Box::pin(async move {
            if wait_ms.unwrap_or(0) > 0
                && self.read_snapshot(after_seq, max_bytes).chunks.is_empty()
            {
                let mut wake_rx = self.wake_tx.subscribe();
                let _ = timeout(
                    Duration::from_millis(wait_ms.unwrap_or(0)),
                    wake_rx.changed(),
                )
                .await;
            }

            Ok(self.read_snapshot(after_seq, max_bytes))
        })
    }

    fn write(&self, chunk: Vec<u8>) -> ExecProcessFuture<'_, WriteResponse> {
        let process_handle = self.process_id.as_str().to_string();
        spawn_local_process_request(async move {
            let params = json!({
                "processHandle": process_handle,
                "deltaBase64": BASE64_STANDARD.encode(chunk),
            });
            match host_request_json::<serde_json::Value, _>("process/writeStdin", &params).await {
                Ok(_) => Ok(WriteResponse {
                    status: WriteStatus::Accepted,
                }),
                Err(_) => Ok(WriteResponse {
                    status: WriteStatus::UnknownProcess,
                }),
            }
        })
    }

    fn signal(&self, signal: ProcessSignal) -> ExecProcessFuture<'_, ()> {
        let process_handle = self.process_id.as_str().to_string();
        spawn_local_process_request(async move {
            let params = json!({
                "processHandle": process_handle,
                "signal": signal,
            });
            let _ = host_request_json::<serde_json::Value, _>("process/signal", &params).await;
            Ok(())
        })
    }

    fn terminate(&self) -> ExecProcessFuture<'_, ()> {
        let process_handle = self.process_id.as_str().to_string();
        spawn_local_process_request(async move {
            let params = json!({ "processHandle": process_handle });
            let _ = host_request_json::<serde_json::Value, _>("process/kill", &params).await;
            Ok(())
        })
    }
}

fn spawn_local_process_request<T>(
    request: impl std::future::Future<Output = Result<T, ExecServerError>> + 'static,
) -> ExecProcessFuture<'static, T>
where
    T: Send + 'static,
{
    let (result_tx, result_rx) = oneshot::channel();
    wasm_bindgen_futures::spawn_local(async move {
        let _ = result_tx.send(request.await);
    });
    Box::pin(async move {
        result_rx.await.map_err(|_| {
            ExecServerError::Protocol("browser host process request was cancelled".to_string())
        })?
    })
}

impl WasmHostExecProcess {
    fn read_snapshot(&self, after_seq: Option<u64>, max_bytes: Option<usize>) -> ReadResponse {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut bytes = 0usize;
        let chunks = state
            .output
            .iter()
            .filter(|chunk| after_seq.is_none_or(|after| chunk.seq > after))
            .take_while(|chunk| {
                if let Some(max_bytes) = max_bytes {
                    if bytes >= max_bytes {
                        return false;
                    }
                }
                bytes = bytes.saturating_add(chunk.chunk.0.len());
                true
            })
            .cloned()
            .collect();

        ReadResponse {
            chunks,
            next_seq: state.next_seq,
            exited: state.exit_code.is_some(),
            exit_code: state.exit_code,
            closed: state.closed,
            failure: state.failure.clone(),
            sandbox_denied: false,
        }
    }

    fn finish_from_host_result(&self, result: HostProcessExited) {
        if !result.stdout.is_empty() {
            self.push_output(ExecOutputStream::Stdout, result.stdout.into_bytes());
        }
        if !result.stderr.is_empty() {
            self.push_output(ExecOutputStream::Stderr, result.stderr.into_bytes());
        }
        self.exit(result.exit_code);
    }

    fn fail(&self, message: String) {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.closed {
                return;
            }
            state.failure = Some(message.clone());
            state.closed = true;
            let seq = state.next_seq;
            state.next_seq += 1;
            let _ = self.wake_tx.send(seq);
        }
        unregister_wasm_host_process(self.process_id.as_str());
        self.events.publish(ExecProcessEvent::Failed(message));
    }

    fn push_output(&self, stream: ExecOutputStream, bytes: Vec<u8>) {
        let chunk = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let seq = state.next_seq;
            state.next_seq += 1;
            let chunk = ProcessOutputChunk {
                seq,
                stream,
                chunk: ByteChunk(bytes),
            };
            state.retained_bytes += chunk.chunk.0.len();
            state.output.push_back(chunk.clone());
            while state.output.len() > RETAINED_OUTPUT_EVENTS_PER_PROCESS
                || state.retained_bytes > RETAINED_OUTPUT_BYTES_PER_PROCESS
            {
                let Some(evicted) = state.output.pop_front() else {
                    break;
                };
                state.retained_bytes = state.retained_bytes.saturating_sub(evicted.chunk.0.len());
            }
            let _ = self.wake_tx.send(seq);
            chunk
        };
        self.events.publish(ExecProcessEvent::Output(chunk));
    }

    fn exit(&self, exit_code: i32) {
        let (exit_seq, closed_seq) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.closed {
                return;
            }
            let exit_seq = state.next_seq;
            state.next_seq += 1;
            state.exit_code = Some(exit_code);
            let closed_seq = state.next_seq;
            state.next_seq += 1;
            state.closed = true;
            let _ = self.wake_tx.send(closed_seq);
            (exit_seq, closed_seq)
        };
        self.events.publish(ExecProcessEvent::Exited {
            seq: exit_seq,
            exit_code,
            sandbox_denied: None,
        });
        self.events
            .publish(ExecProcessEvent::Closed { seq: closed_seq });
        unregister_wasm_host_process(self.process_id.as_str());
    }
}

pub fn handle_wasm_host_process_event(data: JsValue) -> bool {
    let Ok(envelope) = serde_wasm_bindgen::from_value::<HostProcessEventEnvelope>(data) else {
        return false;
    };
    match envelope.event.as_str() {
        "process/outputDelta" => {
            let Ok(params) =
                serde_json::from_value::<HostProcessOutputDelta>(envelope.params.clone())
            else {
                return false;
            };
            let Some(process) = lookup_wasm_host_process(&params.process_handle) else {
                return false;
            };
            let stream = match params.stream.as_str() {
                "stdout" => ExecOutputStream::Stdout,
                "stderr" => ExecOutputStream::Stderr,
                _ => return true,
            };
            match BASE64_STANDARD.decode(params.delta_base64) {
                Ok(bytes) => process.push_output(stream, bytes),
                Err(error) => process.fail(format!("invalid host process output delta: {error}")),
            }
            true
        }
        "process/exited" => {
            let Ok(params) = serde_json::from_value::<HostProcessExited>(envelope.params.clone())
            else {
                return false;
            };
            if let Some(process) = lookup_wasm_host_process(&params.process_handle) {
                process.finish_from_host_result(params);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn register_wasm_host_process(process: &Arc<WasmHostExecProcess>) {
    WASM_HOST_PROCESSES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(
            process.process_id.as_str().to_string(),
            Arc::downgrade(process),
        );
}

fn unregister_wasm_host_process(process_handle: &str) {
    WASM_HOST_PROCESSES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(process_handle);
}

fn lookup_wasm_host_process(process_handle: &str) -> Option<Arc<WasmHostExecProcess>> {
    let mut registry = WASM_HOST_PROCESSES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(process) = registry.get(process_handle).and_then(Weak::upgrade) else {
        registry.remove(process_handle);
        return None;
    };
    Some(process)
}

/// Converts a `PathUri` to the plain path string the almostnode JS host expects.
fn path_uri_to_host_string(path: &PathUri) -> String {
    match path.to_abs_path() {
        Ok(abs) => abs.as_path().to_string_lossy().into_owned(),
        Err(_) => path.to_string(),
    }
}

fn spawn_local_file_system_request<T>(
    request: impl std::future::Future<Output = io::Result<T>> + 'static,
) -> ExecutorFileSystemFuture<'static, T>
where
    T: Send + 'static,
{
    let (result_tx, result_rx) = oneshot::channel();
    wasm_bindgen_futures::spawn_local(async move {
        let _ = result_tx.send(request.await);
    });
    Box::pin(async move {
        result_rx.await.map_err(|_| {
            io::Error::new(
                io::ErrorKind::Interrupted,
                "browser host filesystem request was cancelled",
            )
        })?
    })
}

impl ExecutorFileSystem for WasmHostFileSystem {
    fn canonicalize<'a>(
        &'a self,
        path: &'a PathUri,
        _sandbox: Option<&'a FileSystemSandboxContext>,
    ) -> ExecutorFileSystemFuture<'a, PathUri> {
        Box::pin(async move { Ok(path.clone()) })
    }

    fn read_file<'a>(
        &'a self,
        path: &'a PathUri,
        _sandbox: Option<&'a FileSystemSandboxContext>,
    ) -> ExecutorFileSystemFuture<'a, Vec<u8>> {
        let path = path_uri_to_host_string(path);
        spawn_local_file_system_request(async move {
            let params = json!({ "path": path, "encoding": "base64" });
            let response: HostReadFileResult = host_request_json_io("fs/readFile", &params).await?;
            if response.encoding == "base64" {
                BASE64_STANDARD
                    .decode(response.content)
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
            } else {
                Ok(response.content.into_bytes())
            }
        })
    }

    fn read_file_stream<'a>(
        &'a self,
        path: &'a PathUri,
        sandbox: Option<&'a FileSystemSandboxContext>,
    ) -> ExecutorFileSystemFuture<'a, FileSystemReadStream> {
        Box::pin(async move {
            let bytes = self.read_file(path, sandbox).await?;
            let stream = futures::stream::once(async move { Ok(Bytes::from(bytes)) });
            Ok(FileSystemReadStream::new(stream))
        })
    }

    fn write_file<'a>(
        &'a self,
        path: &'a PathUri,
        contents: Vec<u8>,
        _sandbox: Option<&'a FileSystemSandboxContext>,
    ) -> ExecutorFileSystemFuture<'a, ()> {
        let path = path_uri_to_host_string(path);
        spawn_local_file_system_request(async move {
            let params = json!({
                "path": path,
                "content": BASE64_STANDARD.encode(contents),
                "encoding": "base64",
            });
            let _: serde_json::Value = host_request_json_io("fs/writeFile", &params).await?;
            Ok(())
        })
    }

    fn create_directory<'a>(
        &'a self,
        path: &'a PathUri,
        options: CreateDirectoryOptions,
        _sandbox: Option<&'a FileSystemSandboxContext>,
    ) -> ExecutorFileSystemFuture<'a, ()> {
        let path = path_uri_to_host_string(path);
        spawn_local_file_system_request(async move {
            let params = json!({ "path": path, "recursive": options.recursive });
            let _: serde_json::Value = host_request_json_io("fs/createDirectory", &params).await?;
            Ok(())
        })
    }

    fn get_metadata<'a>(
        &'a self,
        path: &'a PathUri,
        _sandbox: Option<&'a FileSystemSandboxContext>,
    ) -> ExecutorFileSystemFuture<'a, FileMetadata> {
        let path = path_uri_to_host_string(path);
        spawn_local_file_system_request(async move {
            let params = json!({ "path": path });
            let response: HostMetadataResult =
                host_request_json_io("fs/getMetadata", &params).await?;
            Ok(FileMetadata {
                is_directory: response.r#type == "directory",
                is_file: response.r#type == "file",
                is_symlink: false,
                size: response.size,
                created_at_ms: 0,
                modified_at_ms: response.mtime_ms as i64,
            })
        })
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a PathUri,
        _sandbox: Option<&'a FileSystemSandboxContext>,
    ) -> ExecutorFileSystemFuture<'a, Vec<ReadDirectoryEntry>> {
        let path = path_uri_to_host_string(path);
        spawn_local_file_system_request(async move {
            let params = json!({ "path": path });
            let response: HostReadDirectoryResult =
                host_request_json_io("fs/readDirectory", &params).await?;
            Ok(response
                .entries
                .into_iter()
                .map(|entry| ReadDirectoryEntry {
                    file_name: entry.name,
                    is_directory: entry.r#type == "directory",
                    is_file: entry.r#type != "directory",
                })
                .collect())
        })
    }

    fn remove<'a>(
        &'a self,
        _path: &'a PathUri,
        _options: RemoveOptions,
        _sandbox: Option<&'a FileSystemSandboxContext>,
    ) -> ExecutorFileSystemFuture<'a, ()> {
        Box::pin(async move {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "browser host filesystem remove is not wired yet",
            ))
        })
    }

    fn copy<'a>(
        &'a self,
        _source_path: &'a PathUri,
        _destination_path: &'a PathUri,
        _options: CopyOptions,
        _sandbox: Option<&'a FileSystemSandboxContext>,
    ) -> ExecutorFileSystemFuture<'a, ()> {
        Box::pin(async move {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "browser host filesystem copy is not wired yet",
            ))
        })
    }
}

impl HttpClient for WasmHostHttpClient {
    fn http_request(
        &self,
        params: HttpRequestParams,
    ) -> LocalBoxFuture<'_, Result<HttpRequestResponse, ExecServerError>> {
        async move { host_fetch(params).await.map(|(response, _body)| response) }.boxed_local()
    }

    fn http_request_stream(
        &self,
        params: HttpRequestParams,
    ) -> LocalBoxFuture<'_, Result<(HttpRequestResponse, HttpResponseBodyStream), ExecServerError>>
    {
        async move {
            let (response, body) = host_fetch(params).await?;
            Ok((response, HttpResponseBodyStream::buffered(body)))
        }
        .boxed_local()
    }
}

async fn host_fetch(
    params: HttpRequestParams,
) -> Result<(HttpRequestResponse, Vec<u8>), ExecServerError> {
    let headers = params
        .headers
        .iter()
        .map(|header| (header.name.clone(), header.value.clone()))
        .collect::<HashMap<_, _>>();
    let host_params = json!({
        "url": params.url,
        "method": params.method,
        "headers": headers,
        "bodyBase64": params.body.map(|body| BASE64_STANDARD.encode(body.into_inner())),
        "retryOnTailscaleRecovery": true,
    });
    let response: HostFetchResult = host_request_json("network/fetch", &host_params).await?;
    let body = BASE64_STANDARD
        .decode(response.body_base64)
        .map_err(|err| ExecServerError::Protocol(format!("invalid host fetch body: {err}")))?;
    let headers = response
        .headers
        .into_iter()
        .map(|(name, value)| HttpHeader { name, value })
        .collect();
    Ok((
        HttpRequestResponse {
            status: response.status,
            headers,
            body: ByteChunk(body.clone()),
        },
        body,
    ))
}

async fn host_request_json_io<T, P>(op: &str, params: &P) -> io::Result<T>
where
    T: DeserializeOwned,
    P: Serialize + ?Sized,
{
    host_request_json(op, params)
        .await
        .map_err(exec_error_to_io)
}

async fn host_request_json<T, P>(op: &str, params: &P) -> Result<T, ExecServerError>
where
    T: DeserializeOwned,
    P: Serialize + ?Sized,
{
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    let params = params
        .serialize(&serializer)
        .map_err(|err| ExecServerError::Protocol(format!("host request encode failed: {err}")))?;
    let promise = almostnode_codex_host_request(op, params).map_err(js_error_to_exec_error)?;
    let result = JsFuture::from(promise)
        .await
        .map_err(js_error_to_exec_error)?;
    serde_wasm_bindgen::from_value(result)
        .map_err(|err| ExecServerError::Protocol(format!("host response decode failed: {err}")))
}

fn js_error_to_exec_error(error: JsValue) -> ExecServerError {
    let message = js_error_to_string(error.clone());
    if let Some(code) = js_error_property(&error, "code") {
        return ExecServerError::Protocol(format!("{code}: {message}"));
    }
    ExecServerError::Protocol(message)
}

fn exec_error_to_io(error: ExecServerError) -> io::Error {
    let message = error.to_string();
    let kind = if message.contains("ENOENT:")
        || message.contains("ENOENT")
        || message.contains("NotFound")
    {
        io::ErrorKind::NotFound
    } else {
        io::ErrorKind::Other
    };
    io::Error::new(kind, message)
}

fn js_error_property(error: &JsValue, name: &str) -> Option<String> {
    Reflect::get(error, &JsValue::from_str(name))
        .ok()
        .and_then(|value| value.as_string())
}

fn js_error_to_string(error: JsValue) -> String {
    if let Some(message) = error.as_string() {
        return message;
    }
    js_sys::JSON::stringify(&error)
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| "unknown JavaScript host error".to_string())
}
