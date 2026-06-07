use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::AtomicBool;

use anyhow::Result;
use anyhow::bail;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

const HOST_PROCESS_SHIM_REQUIRED: &str =
    "browser process spawning requires an almostnode host exec/process shim";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalSize {
    pub rows: u16,
    pub cols: u16,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self { rows: 24, cols: 80 }
    }
}

pub struct ProcessHandle {
    writer_tx: StdMutex<Option<mpsc::Sender<Vec<u8>>>>,
    exit_status: Arc<AtomicBool>,
    exit_code: Arc<StdMutex<Option<i32>>>,
}

impl std::fmt::Debug for ProcessHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessHandle").finish()
    }
}

impl ProcessHandle {
    fn new(
        writer_tx: mpsc::Sender<Vec<u8>>,
        exit_status: Arc<AtomicBool>,
        exit_code: Arc<StdMutex<Option<i32>>>,
    ) -> Self {
        Self {
            writer_tx: StdMutex::new(Some(writer_tx)),
            exit_status,
            exit_code,
        }
    }

    pub fn writer_sender(&self) -> mpsc::Sender<Vec<u8>> {
        if let Ok(writer_tx) = self.writer_tx.lock()
            && let Some(writer_tx) = writer_tx.as_ref()
        {
            return writer_tx.clone();
        }

        let (writer_tx, writer_rx) = mpsc::channel(1);
        drop(writer_rx);
        writer_tx
    }

    pub fn has_exited(&self) -> bool {
        self.exit_status.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code.lock().ok().and_then(|guard| *guard)
    }

    pub fn resize(&self, _size: TerminalSize) -> anyhow::Result<()> {
        bail!("browser PTY resize requires an almostnode terminal host shim")
    }

    pub fn close_stdin(&self) {
        if let Ok(mut writer_tx) = self.writer_tx.lock() {
            writer_tx.take();
        }
    }

    pub fn request_terminate(&self) {
        self.exit_status
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn terminate(&self) {
        self.request_terminate();
        self.close_stdin();
    }
}

#[derive(Debug)]
pub struct SpawnedProcess {
    pub session: ProcessHandle,
    pub stdout_rx: mpsc::Receiver<Vec<u8>>,
    pub stderr_rx: mpsc::Receiver<Vec<u8>>,
    pub exit_rx: oneshot::Receiver<i32>,
}

type ResizeFn = Box<dyn FnMut(TerminalSize) -> anyhow::Result<()> + Send>;

pub struct ProcessDriver {
    pub writer_tx: mpsc::Sender<Vec<u8>>,
    pub stdout_rx: broadcast::Receiver<Vec<u8>>,
    pub stderr_rx: Option<broadcast::Receiver<Vec<u8>>>,
    pub exit_rx: oneshot::Receiver<i32>,
    pub terminator: Option<Box<dyn FnMut() + Send + Sync>>,
    pub writer_handle: Option<JoinHandle<()>>,
    pub resizer: Option<ResizeFn>,
}

pub fn combine_output_receivers(
    _stdout_rx: mpsc::Receiver<Vec<u8>>,
    _stderr_rx: mpsc::Receiver<Vec<u8>>,
) -> broadcast::Receiver<Vec<u8>> {
    let (_tx, rx) = broadcast::channel(1);
    rx
}

pub fn spawn_from_driver(driver: ProcessDriver) -> SpawnedProcess {
    let ProcessDriver {
        writer_tx,
        stdout_rx,
        stderr_rx,
        exit_rx,
        terminator,
        writer_handle,
        resizer,
    } = driver;
    let _ = (stdout_rx, stderr_rx, terminator, writer_handle, resizer);

    let (_stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>(1);
    let (_stderr_tx, stderr_rx) = mpsc::channel::<Vec<u8>>(1);
    let exit_status = Arc::new(AtomicBool::new(false));
    let exit_code = Arc::new(StdMutex::new(None));
    let session = ProcessHandle::new(writer_tx, exit_status, exit_code);

    SpawnedProcess {
        session,
        stdout_rx,
        stderr_rx,
        exit_rx,
    }
}

async fn unsupported_spawn() -> Result<SpawnedProcess> {
    bail!(HOST_PROCESS_SHIM_REQUIRED)
}

pub async fn spawn_pipe_process(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &HashMap<String, String>,
    arg0: &Option<String>,
) -> Result<SpawnedProcess> {
    let _ = (program, args, cwd, env, arg0);
    unsupported_spawn().await
}

pub async fn spawn_pipe_process_no_stdin(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &HashMap<String, String>,
    arg0: &Option<String>,
) -> Result<SpawnedProcess> {
    let _ = (program, args, cwd, env, arg0);
    unsupported_spawn().await
}

pub async fn spawn_pty_process(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &HashMap<String, String>,
    arg0: &Option<String>,
    size: TerminalSize,
) -> Result<SpawnedProcess> {
    let _ = (program, args, cwd, env, arg0, size);
    unsupported_spawn().await
}

pub fn conpty_supported() -> bool {
    true
}

pub type ExecCommandSession = ProcessHandle;
pub type SpawnedPty = SpawnedProcess;

pub mod pipe {
    use super::*;

    pub async fn spawn_process(
        program: &str,
        args: &[String],
        cwd: &Path,
        env: &HashMap<String, String>,
        arg0: &Option<String>,
    ) -> Result<SpawnedProcess> {
        super::spawn_pipe_process(program, args, cwd, env, arg0).await
    }

    pub async fn spawn_process_no_stdin(
        program: &str,
        args: &[String],
        cwd: &Path,
        env: &HashMap<String, String>,
        arg0: &Option<String>,
    ) -> Result<SpawnedProcess> {
        super::spawn_pipe_process_no_stdin(program, args, cwd, env, arg0).await
    }

    pub async fn spawn_process_no_stdin_with_inherited_fds(
        program: &str,
        args: &[String],
        cwd: &Path,
        env: &HashMap<String, String>,
        arg0: &Option<String>,
        inherited_fds: &[i32],
    ) -> Result<SpawnedProcess> {
        let _ = inherited_fds;
        super::spawn_pipe_process_no_stdin(program, args, cwd, env, arg0).await
    }
}

pub mod pty {
    use super::*;

    pub fn conpty_supported() -> bool {
        true
    }

    pub async fn spawn_process(
        program: &str,
        args: &[String],
        cwd: &Path,
        env: &HashMap<String, String>,
        arg0: &Option<String>,
        size: TerminalSize,
    ) -> Result<SpawnedProcess> {
        super::spawn_pty_process(program, args, cwd, env, arg0, size).await
    }

    pub async fn spawn_process_with_inherited_fds(
        program: &str,
        args: &[String],
        cwd: &Path,
        env: &HashMap<String, String>,
        arg0: &Option<String>,
        size: TerminalSize,
        inherited_fds: &[i32],
    ) -> Result<SpawnedProcess> {
        let _ = inherited_fds;
        super::spawn_pty_process(program, args, cwd, env, arg0, size).await
    }
}

pub mod process_group {
    use std::io;

    pub fn set_parent_death_signal(_parent_pid: i32) -> io::Result<()> {
        Ok(())
    }

    pub fn detach_from_tty() -> io::Result<()> {
        Ok(())
    }

    pub fn set_process_group() -> io::Result<()> {
        Ok(())
    }

    pub fn kill_process_group_by_pid(_pid: u32) -> io::Result<()> {
        Ok(())
    }

    pub fn terminate_process_group(_process_group_id: u32) -> io::Result<bool> {
        Ok(false)
    }

    pub fn kill_process_group(_process_group_id: u32) -> io::Result<()> {
        Ok(())
    }

    pub fn kill_child_process_group<T>(_child: &mut T) -> io::Result<()> {
        Ok(())
    }
}
