use std::path::PathBuf;
use std::str::FromStr;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use std::sync::Arc;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use std::sync::atomic::AtomicBool;

#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use crate::app_command::AppCommand;
use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use crate::bottom_pane::ChatComposer;
use crate::bottom_pane::InputResult;
use crate::bottom_pane::QueuedInputAction;
use crate::bottom_pane::prompt_args::parse_slash_name;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use crate::chatwidget::ChatWidget;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use crate::chatwidget::ChatWidgetInit;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use crate::chatwidget::CodexOpTarget;
use crate::history_cell::HistoryCell;
use crate::history_cell::PlainHistoryCell;
use crate::history_cell::SessionHeaderHistoryCell;
use crate::history_cell::UserHistoryCell;
use crate::history_cell::new_agent_message;
use crate::history_cell::new_error_event;
use crate::history_cell::new_info_event;
use crate::history_cell::new_plan_update;
use crate::history_cell::raw_lines_from_source;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use crate::legacy_core::config::Config;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use crate::model_catalog::ModelCatalog;
use crate::render::renderable::Renderable;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use crate::session_state::MessageHistoryMetadata;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use crate::session_state::ThreadSessionState;
use crate::slash_command::SlashCommand;
use crate::slash_command::built_in_slash_commands;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use crate::tui::FrameRequester;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use codex_app_server_protocol::ServerNotification;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use codex_app_server_protocol::UserInput;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use codex_protocol::ThreadId;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::plan_tool::UpdatePlanArgs;
#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
use codex_utils_absolute_path::AbsolutePathBuf;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyEventState;
use crossterm::event::KeyModifiers;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::unbounded_channel;

pub const DEFAULT_WIDTH: u16 = 119;
pub const DEFAULT_HEIGHT: u16 = 30;
pub const DEFAULT_MODEL: &str = "gpt-5.5 medium";
const MAX_HEADER_WIDTH: u16 = 70;
const BROWSER_TUI_PLACEHOLDER: &str = "Explain this code base to me";
const INIT_PROMPT: &str = include_str!("../prompt_for_init_command.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTuiFrame {
    pub width: u16,
    pub height: u16,
    pub version: Option<String>,
    pub model: String,
    pub directory: String,
    pub input: String,
    pub placeholder: Option<String>,
    pub status: BrowserTuiStatus,
    pub transcript: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserTuiStatus {
    Ready,
    Thinking,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BrowserTuiSessionOptions {
    pub cwd: Option<String>,
    pub env: Vec<(String, String)>,
    pub terminal_width: Option<u16>,
    pub terminal_height: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTuiRunResult {
    pub ansi: String,
    pub action: BrowserTuiAction,
    pub cursor: Option<BrowserTuiCursorPosition>,
    pub scrollback_ansi: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserTuiCursorPosition {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserTuiAction {
    None,
    Login,
    Exec { prompt: String },
    Shell { command: String },
    Exit { exit_code: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserTuiResultKind {
    Exec,
    Shell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTuiAppendResult {
    pub kind: BrowserTuiResultKind,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserTuiEvent {
    Key(BrowserTuiKeyEvent),
    Paste(String),
    Resize,
    Draw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTuiKeyEvent {
    pub code: BrowserTuiKeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserTuiKeyCode {
    Char(String),
    Enter,
    Backspace,
    Delete,
    Escape,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Unknown(String),
}

pub struct BrowserInteractiveTuiSession {
    directory: String,
    model: String,
    composer: ChatComposer,
    app_event_rx: UnboundedReceiver<AppEvent>,
    history: Vec<Box<dyn HistoryCell>>,
    pending_scrollback_lines: Vec<Line<'static>>,
    status: BrowserTuiStatus,
    #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
    real: Option<RealBrowserInteractiveTuiSession>,
}

impl std::fmt::Debug for BrowserInteractiveTuiSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("BrowserInteractiveTuiSession");
        debug
            .field("directory", &self.directory)
            .field("model", &self.model)
            .field("input", &self.composer.current_text_with_pending())
            .field("history_len", &self.history.len())
            .field("status", &self.status);
        #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
        debug.field("real", &self.real.as_ref().map(|_| "ChatWidget"));
        debug.finish()
    }
}

impl BrowserTuiFrame {
    pub fn new(directory: impl Into<String>) -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            version: browser_tui_version(),
            model: DEFAULT_MODEL.to_string(),
            directory: directory.into(),
            input: String::new(),
            placeholder: Some("Explain this code base to me".to_string()),
            status: BrowserTuiStatus::Ready,
            transcript: Vec::new(),
        }
    }

    pub fn with_dimensions(mut self, width: u16, height: u16) -> Self {
        self.width = width.clamp(40, 240);
        self.height = height.clamp(12, 80);
        self
    }
}

impl BrowserTuiStatus {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "ready" => Ok(Self::Ready),
            "thinking" | "running" => Ok(Self::Thinking),
            _ => Err(format!("unsupported browser TUI status: {value}\n")),
        }
    }
}

pub fn browser_tui_version() -> Option<String> {
    let version = env!("CARGO_PKG_VERSION");
    if version == "0.0.0" {
        None
    } else {
        Some(version.to_string())
    }
}

fn browser_tui_version_for_env(env: &[(String, String)]) -> Option<String> {
    env_value(env, "CODEX_CLI_VERSION")
        .filter(|version| !version.trim().is_empty())
        .map(ToString::to_string)
        .or_else(browser_tui_version)
}

fn browser_tui_has_auth(env: &[(String, String)]) -> bool {
    ["CODEX_ACCESS_TOKEN", "CODEX_API_KEY", "OPENAI_API_KEY"]
        .iter()
        .any(|key| {
            env_value(env, key)
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
        })
}

impl Default for BrowserInteractiveTuiSession {
    fn default() -> Self {
        let (composer, app_event_rx) = new_browser_composer();
        Self {
            directory: "~".to_string(),
            model: DEFAULT_MODEL.to_string(),
            composer,
            app_event_rx,
            history: Vec::new(),
            pending_scrollback_lines: Vec::new(),
            status: BrowserTuiStatus::Ready,
            #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
            real: None,
        }
    }
}

impl BrowserInteractiveTuiSession {
    pub fn start(
        &mut self,
        prompt: Option<String>,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
        {
            let mut real = RealBrowserInteractiveTuiSession::new(options)?;
            let result = real.start(prompt, options);
            self.real = Some(real);
            return result;
        }

        self.directory = options.cwd.clone().unwrap_or_else(|| "~".to_string());
        self.model = env_value(&options.env, "CODEX_MODEL")
            .or_else(|| env_value(&options.env, "OPENAI_MODEL"))
            .unwrap_or(DEFAULT_MODEL)
            .to_string();
        let (composer, app_event_rx) = new_browser_composer();
        self.composer = composer;
        self.app_event_rx = app_event_rx;
        self.composer
            .set_text_content(prompt.unwrap_or_default(), Vec::new(), Vec::new());
        self.history.clear();
        self.pending_scrollback_lines.clear();
        self.status = BrowserTuiStatus::Ready;
        if self.composer.current_text_with_pending().trim().is_empty() {
            let action = if browser_tui_has_auth(&options.env) {
                BrowserTuiAction::None
            } else {
                BrowserTuiAction::Login
            };
            return self.render(options, action);
        }
        self.submit(options)
    }

    pub fn set_input(
        &mut self,
        input: String,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
        if let Some(real) = self.real.as_mut() {
            return real.set_input(input, options);
        }

        self.composer
            .set_text_content(input, Vec::new(), Vec::new());
        self.status = BrowserTuiStatus::Ready;
        self.render(options, BrowserTuiAction::None)
    }

    pub fn submit(
        &mut self,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
        if let Some(real) = self.real.as_mut() {
            return real.submit(options);
        }

        let text = self.composer.current_text_with_pending().trim().to_string();
        if text.is_empty() {
            self.status = BrowserTuiStatus::Ready;
            return self.render(options, BrowserTuiAction::None);
        }

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let (result, _) = self.composer.handle_key_event(key);
        self.apply_input_result(result, options)
    }

    pub fn append_result(
        &mut self,
        result: BrowserTuiAppendResult,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
        if let Some(real) = self.real.as_mut() {
            return real.append_result(result, options);
        }

        self.status = BrowserTuiStatus::Ready;

        if !result.stdout.trim().is_empty() {
            match result.kind {
                BrowserTuiResultKind::Exec => {
                    let cwd = PathBuf::from(&self.directory);
                    self.push_history_cell(Box::new(new_agent_message(
                        result.stdout.trim_end().to_string(),
                        &cwd,
                    )), options);
                }
                BrowserTuiResultKind::Shell => {
                    self.push_history_cell(
                        Box::new(PlainHistoryCell::new(raw_lines_from_source(
                            result.stdout.trim_end(),
                        ))),
                        options,
                    );
                }
            }
        }
        if !result.stderr.trim().is_empty() {
            self.push_history_cell(
                Box::new(new_error_event(result.stderr.trim_end().to_string())),
                options,
            );
        }
        if result.stdout.trim().is_empty()
            && result.stderr.trim().is_empty()
            && result.kind == BrowserTuiResultKind::Shell
        {
            self.push_history_cell(
                Box::new(new_info_event(
                    "(command completed with no output)".to_string(),
                    None,
                )),
                options,
            );
        }
        if result.exit_code != 0 {
            let source = match result.kind {
                BrowserTuiResultKind::Exec => "codex exec",
                BrowserTuiResultKind::Shell => "shell command",
            };
            self.push_history_cell(
                Box::new(new_error_event(format!(
                    "{source} exited with code {}",
                    result.exit_code
                ))),
                options,
            );
        }

        self.render(options, BrowserTuiAction::None)
    }

    pub fn append_agent_message(
        &mut self,
        markdown_source: String,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
        if let Some(real) = self.real.as_mut() {
            return real.append_agent_message(markdown_source, options);
        }

        self.status = BrowserTuiStatus::Ready;
        if !markdown_source.trim().is_empty() {
            let cwd = PathBuf::from(&self.directory);
            self.push_history_cell(Box::new(new_agent_message(markdown_source, &cwd)), options);
        }
        self.render(options, BrowserTuiAction::None)
    }

    pub fn append_plan_update(
        &mut self,
        update: UpdatePlanArgs,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
        if let Some(real) = self.real.as_mut() {
            return real.append_plan_update(update, options);
        }

        self.status = BrowserTuiStatus::Ready;
        self.push_history_cell(Box::new(new_plan_update(update)), options);
        self.render(options, BrowserTuiAction::None)
    }

    pub fn apply_server_notification_json(
        &mut self,
        notification_json: String,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
        if let Some(real) = self.real.as_mut() {
            return real.apply_server_notification_json(notification_json, options);
        }

        let _ = notification_json;
        self.render(options, BrowserTuiAction::None)
    }

    pub fn handle_event(
        &mut self,
        event: BrowserTuiEvent,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        #[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
        if let Some(real) = self.real.as_mut() {
            return real.handle_event(event, options);
        }

        match event {
            BrowserTuiEvent::Key(key) => self.handle_key_event(key, options),
            BrowserTuiEvent::Paste(text) => {
                self.composer.handle_paste(normalize_paste(&text));
                self.status = BrowserTuiStatus::Ready;
                self.render(options, BrowserTuiAction::None)
            }
            BrowserTuiEvent::Resize | BrowserTuiEvent::Draw => {
                self.render(options, BrowserTuiAction::None)
            }
        }
    }

    fn handle_key_event(
        &mut self,
        key: BrowserTuiKeyEvent,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        if key.ctrl
            && matches!(
                &key.code,
                BrowserTuiKeyCode::Char(value) if value.eq_ignore_ascii_case("c")
            )
        {
            self.composer
                .set_text_content(String::new(), Vec::new(), Vec::new());
            self.status = BrowserTuiStatus::Ready;
            return self.render(options, BrowserTuiAction::Exit { exit_code: 130 });
        }

        if key.ctrl
            && matches!(
                &key.code,
                BrowserTuiKeyCode::Char(value) if value.eq_ignore_ascii_case("d")
            )
        {
            self.composer
                .set_text_content(String::new(), Vec::new(), Vec::new());
            self.status = BrowserTuiStatus::Ready;
            return self.render(options, BrowserTuiAction::Exit { exit_code: 0 });
        }

        let Some(key_event) = browser_key_event_to_crossterm(key) else {
            return self.render(options, BrowserTuiAction::None);
        };
        let (result, _) = self.composer.handle_key_event(key_event);
        self.apply_input_result(result, options)
    }

    fn apply_input_result(
        &mut self,
        result: InputResult,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        self.drain_app_events(options);
        let action = match result {
            InputResult::Submitted { text, .. } => self.action_for_submitted_text(text, options),
            InputResult::Queued { text, action, .. } => {
                self.action_for_queued_text(text, action, options)
            }
            InputResult::Command(command) => {
                self.action_for_slash_text(format!("/{}", command.command()), options)
            }
            InputResult::CommandWithArgs(command, args, _) => {
                self.action_for_slash_text(format!("/{} {args}", command.command()), options)
            }
            InputResult::ServiceTierCommand(command) => {
                self.status = BrowserTuiStatus::Ready;
                self.push_history_cell(
                    Box::new(new_info_event(
                        format!(
                            "The native /model service-tier command '{}' is recognized, but the browser settings bridge is not wired yet.",
                            command.name
                        ),
                        None,
                    )),
                    options,
                );
                BrowserTuiAction::None
            }
            InputResult::None => {
                self.status = BrowserTuiStatus::Ready;
                BrowserTuiAction::None
            }
        };
        self.render(options, action)
    }

    fn action_for_queued_text(
        &mut self,
        text: String,
        queued_action: QueuedInputAction,
        options: &BrowserTuiSessionOptions,
    ) -> BrowserTuiAction {
        match queued_action {
            QueuedInputAction::RunShell => self.action_for_shell_text(text, options),
            QueuedInputAction::ParseSlash => self.action_for_slash_text(text, options),
            QueuedInputAction::Plain => self.action_for_submitted_text(text, options),
        }
    }

    fn action_for_submitted_text(
        &mut self,
        text: String,
        options: &BrowserTuiSessionOptions,
    ) -> BrowserTuiAction {
        if text.trim_start().starts_with('!') {
            return self.action_for_shell_text(text, options);
        }

        let prompt = text.trim().to_string();
        if prompt.is_empty() {
            self.status = BrowserTuiStatus::Ready;
            return BrowserTuiAction::None;
        }

        self.push_history_cell(Box::new(new_user_history_cell(prompt.clone())), options);
        if !browser_tui_has_auth(&options.env) {
            self.status = BrowserTuiStatus::Ready;
            return BrowserTuiAction::Login;
        }

        self.status = BrowserTuiStatus::Thinking;
        BrowserTuiAction::Exec { prompt }
    }

    fn action_for_shell_text(
        &mut self,
        text: String,
        options: &BrowserTuiSessionOptions,
    ) -> BrowserTuiAction {
        let command = text
            .trim_start()
            .strip_prefix('!')
            .unwrap_or(text.as_str())
            .trim()
            .to_string();
        self.push_history_cell(Box::new(new_user_history_cell(format!("!{command}"))), options);
        if command.is_empty() {
            self.push_history_cell(
                Box::new(new_error_event("No shell command provided.".to_string())),
                options,
            );
            self.status = BrowserTuiStatus::Ready;
            BrowserTuiAction::None
        } else {
            self.status = BrowserTuiStatus::Thinking;
            BrowserTuiAction::Shell { command }
        }
    }

    fn action_for_slash_text(
        &mut self,
        text: String,
        options: &BrowserTuiSessionOptions,
    ) -> BrowserTuiAction {
        let Some(submission) = browser_tui_slash_submission(&text, &self.model, &self.directory)
        else {
            return self.action_for_submitted_text(text, options);
        };

        self.status = BrowserTuiStatus::Ready;
        match submission {
            BrowserTuiSlashSubmission::Exit { exit_code } => BrowserTuiAction::Exit { exit_code },
            BrowserTuiSlashSubmission::Clear => {
                self.history.clear();
                self.pending_scrollback_lines.clear();
                BrowserTuiAction::None
            }
            BrowserTuiSlashSubmission::Message(message) => {
                self.push_history_cell(Box::new(new_info_event(message, None)), options);
                BrowserTuiAction::None
            }
            BrowserTuiSlashSubmission::Exec { command, prompt } => {
                self.push_history_cell(
                    Box::new(new_user_history_cell(format!("/{command}"))),
                    options,
                );
                if !browser_tui_has_auth(&options.env) {
                    return BrowserTuiAction::Login;
                }
                self.status = BrowserTuiStatus::Thinking;
                BrowserTuiAction::Exec { prompt }
            }
        }
    }

    fn drain_app_events(&mut self, options: &BrowserTuiSessionOptions) {
        while let Ok(event) = self.app_event_rx.try_recv() {
            if let AppEvent::InsertHistoryCell(cell) = event {
                self.push_history_cell(cell, options);
            }
        }
    }

    fn push_history_cell(
        &mut self,
        cell: Box<dyn HistoryCell>,
        options: &BrowserTuiSessionOptions,
    ) {
        let width = browser_tui_width(options);
        self.pending_scrollback_lines.extend(cell.display_lines(width));
        self.history.push(cell);
    }

    fn render(
        &mut self,
        options: &BrowserTuiSessionOptions,
        action: BrowserTuiAction,
    ) -> Result<BrowserTuiRunResult, String> {
        self.composer.flush_paste_burst_if_due();
        let width = options
            .terminal_width
            .unwrap_or(DEFAULT_WIDTH)
            .clamp(40, 240);
        let height = options
            .terminal_height
            .unwrap_or(DEFAULT_HEIGHT)
            .clamp(12, 80);
        let frame = render_browser_tui_session(BrowserTuiSessionRender {
            width,
            height,
            version: browser_tui_version_for_env(&options.env),
            model: self.model.clone(),
            directory: self.directory.clone(),
            status: self.status,
            history: &self.history,
            composer: &self.composer,
        })?;
        Ok(BrowserTuiRunResult {
            ansi: frame.ansi,
            action,
            cursor: frame.cursor,
            scrollback_ansi: take_scrollback_ansi(&mut self.pending_scrollback_lines, width),
        })
    }
}

#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
struct RealBrowserInteractiveTuiSession {
    chat: ChatWidget,
    app_event_rx: UnboundedReceiver<AppEvent>,
    op_rx: UnboundedReceiver<AppCommand>,
    transcript_cells: Vec<Arc<dyn HistoryCell>>,
    pending_scrollback_lines: Vec<Line<'static>>,
    cwd: PathBuf,
    has_auth: bool,
}

#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
impl RealBrowserInteractiveTuiSession {
    fn new(options: &BrowserTuiSessionOptions) -> Result<Self, String> {
        let cwd = options
            .cwd
            .clone()
            .unwrap_or_else(|| "/project".to_string());
        let cwd_abs = AbsolutePathBuf::resolve_path_against_base(cwd.as_str(), "/");
        let model = env_value(&options.env, "CODEX_MODEL")
            .or_else(|| env_value(&options.env, "OPENAI_MODEL"))
            .unwrap_or("gpt-5.5")
            .to_string();

        let mut config = Config::default();
        config.cwd = cwd_abs.clone();
        config.workspace_roots = vec![cwd_abs.clone()];
        config
            .permissions
            .set_workspace_roots(vec![cwd_abs.clone()]);
        config.model = Some(model.clone());
        config.model_reasoning_effort = Some(ReasoningEffort::Medium);
        config.disable_paste_burst = true;

        let (app_event_tx_raw, app_event_rx) = unbounded_channel::<AppEvent>();
        let app_event_tx = AppEventSender::new(app_event_tx_raw);
        let (op_tx, op_rx) = unbounded_channel::<AppCommand>();
        let has_auth = browser_tui_has_auth(&options.env);
        let common = ChatWidgetInit {
            config: config.clone(),
            frame_requester: FrameRequester::default(),
            app_event_tx,
            workspace_command_runner: None,
            initial_user_message: None,
            enhanced_keys_supported: false,
            has_chatgpt_account: has_auth,
            model_catalog: Arc::new(ModelCatalog::new(Vec::new())),
            feedback: codex_feedback::CodexFeedback::new(),
            is_first_run: false,
            status_account_display: None,
            runtime_model_provider_base_url: None,
            initial_plan_type: None,
            model: Some(model.clone()),
            startup_tooltip_override: None,
            status_line_invalid_items_warned: Arc::new(AtomicBool::new(false)),
            terminal_title_invalid_items_warned: Arc::new(AtomicBool::new(false)),
            session_telemetry: codex_otel::SessionTelemetry::default(),
        };
        let mut chat = ChatWidget::new_with_op_target(common, CodexOpTarget::Direct(op_tx));
        chat.set_model(&model);
        chat.handle_thread_session(ThreadSessionState {
            thread_id: ThreadId::new(),
            forked_from_id: None,
            fork_parent_title: None,
            thread_name: None,
            model,
            model_provider_id: config.model_provider_id.clone(),
            service_tier: config.service_tier.clone(),
            approval_policy: codex_app_server_protocol::AskForApproval::OnRequest,
            approvals_reviewer: config.approvals_reviewer,
            permission_profile: config.permissions.effective_permission_profile(),
            active_permission_profile: config.permissions.active_permission_profile(),
            cwd: cwd_abs.clone(),
            runtime_workspace_roots: vec![cwd_abs],
            instruction_source_paths: Vec::new(),
            reasoning_effort: Some(ReasoningEffort::Medium),
            collaboration_mode: None,
            personality: None,
            message_history: Some(MessageHistoryMetadata::default()),
            network_proxy: None,
            rollout_path: None,
        });

        let mut session = Self {
            chat,
            app_event_rx,
            op_rx,
            transcript_cells: Vec::new(),
            pending_scrollback_lines: Vec::new(),
            cwd: PathBuf::from(cwd),
            has_auth,
        };
        session.drain_real_events(browser_tui_width(options));
        Ok(session)
    }

    fn start(
        &mut self,
        prompt: Option<String>,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        let prompt = prompt.unwrap_or_default();
        if !prompt.trim().is_empty() {
            self.chat.set_composer_text(prompt, Vec::new(), Vec::new());
            return self.submit(options);
        }
        let action = if self.has_auth {
            BrowserTuiAction::None
        } else {
            BrowserTuiAction::Login
        };
        self.finish(options, action)
    }

    fn set_input(
        &mut self,
        input: String,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        self.chat.set_composer_text(input, Vec::new(), Vec::new());
        self.finish(options, BrowserTuiAction::None)
    }

    fn submit(
        &mut self,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        self.chat
            .handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        self.finish(options, BrowserTuiAction::None)
    }

    fn append_result(
        &mut self,
        result: BrowserTuiAppendResult,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        if !result.stdout.trim().is_empty() {
            match result.kind {
                BrowserTuiResultKind::Exec => {
                    self.chat.add_to_history(new_agent_message(
                        result.stdout.trim_end().to_string(),
                        &self.cwd,
                    ));
                }
                BrowserTuiResultKind::Shell => {
                    self.chat
                        .add_to_history(PlainHistoryCell::new(raw_lines_from_source(
                            result.stdout.trim_end(),
                        )));
                }
            }
        }
        if !result.stderr.trim().is_empty() {
            self.chat
                .add_to_history(new_error_event(result.stderr.trim_end().to_string()));
        }
        if result.stdout.trim().is_empty()
            && result.stderr.trim().is_empty()
            && result.kind == BrowserTuiResultKind::Shell
        {
            self.chat.add_to_history(new_info_event(
                "(command completed with no output)".to_string(),
                None,
            ));
        }
        if result.exit_code != 0 {
            let source = match result.kind {
                BrowserTuiResultKind::Exec => "codex exec",
                BrowserTuiResultKind::Shell => "shell command",
            };
            self.chat.add_to_history(new_error_event(format!(
                "{source} exited with code {}",
                result.exit_code
            )));
        }
        self.finish(options, BrowserTuiAction::None)
    }

    fn append_agent_message(
        &mut self,
        markdown_source: String,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        if !markdown_source.trim().is_empty() {
            self.chat
                .add_to_history(new_agent_message(markdown_source, &self.cwd));
        }
        self.finish(options, BrowserTuiAction::None)
    }

    fn append_plan_update(
        &mut self,
        update: UpdatePlanArgs,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        self.chat.add_to_history(new_plan_update(update));
        self.finish(options, BrowserTuiAction::None)
    }

    fn apply_server_notification_json(
        &mut self,
        notification_json: String,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        let notification = serde_json::from_str::<ServerNotification>(&notification_json)
            .map_err(|err| format!("invalid browser TUI server notification: {err}"))?;
        self.chat
            .handle_server_notification(notification, /*replay_kind*/ None);
        self.finish(options, BrowserTuiAction::None)
    }

    fn handle_event(
        &mut self,
        event: BrowserTuiEvent,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRunResult, String> {
        match event {
            BrowserTuiEvent::Key(key) => {
                if let Some(key_event) = browser_key_event_to_crossterm(key) {
                    self.chat.handle_key_event(key_event);
                }
            }
            BrowserTuiEvent::Paste(text) => self.chat.handle_paste(normalize_paste(&text)),
            BrowserTuiEvent::Resize | BrowserTuiEvent::Draw => {}
        }
        self.finish(options, BrowserTuiAction::None)
    }

    fn finish(
        &mut self,
        options: &BrowserTuiSessionOptions,
        fallback_action: BrowserTuiAction,
    ) -> Result<BrowserTuiRunResult, String> {
        self.has_auth = browser_tui_has_auth(&options.env);
        let width = browser_tui_width(options);
        let action = self.drain_real_events(width).unwrap_or(fallback_action);
        let frame = self.render(options)?;
        Ok(BrowserTuiRunResult {
            ansi: frame.ansi,
            action,
            cursor: frame.cursor,
            scrollback_ansi: take_scrollback_ansi(&mut self.pending_scrollback_lines, width),
        })
    }

    fn drain_real_events(&mut self, scrollback_width: u16) -> Option<BrowserTuiAction> {
        let mut action = None;
        while let Ok(event) = self.app_event_rx.try_recv() {
            match event {
                AppEvent::InsertHistoryCell(cell) => {
                    self.pending_scrollback_lines
                        .extend(cell.display_lines(scrollback_width));
                    self.transcript_cells.push(Arc::from(cell));
                }
                AppEvent::ClearUi => {
                    self.transcript_cells.clear();
                    self.pending_scrollback_lines.clear();
                }
                AppEvent::Exit(_) => {
                    action = Some(BrowserTuiAction::Exit { exit_code: 0 });
                }
                AppEvent::CodexOp(op) | AppEvent::SubmitThreadOp { op, .. } => {
                    action = self.action_from_app_command(op).or(action);
                }
                AppEvent::ClearUiAndSubmitUserMessage { text } => {
                    self.transcript_cells.clear();
                    self.chat.set_composer_text(text, Vec::new(), Vec::new());
                    self.chat
                        .handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
                }
                _ => {}
            }
        }
        while let Ok(op) = self.op_rx.try_recv() {
            action = self.action_from_app_command(op).or(action);
        }
        action
    }

    fn action_from_app_command(&mut self, op: AppCommand) -> Option<BrowserTuiAction> {
        match op {
            AppCommand::RunUserShellCommand { command } => {
                Some(BrowserTuiAction::Shell { command })
            }
            AppCommand::UserTurn { items, .. } => {
                let prompt = prompt_from_user_inputs(&items);
                if prompt.trim().is_empty() {
                    None
                } else if self.has_auth {
                    Some(BrowserTuiAction::Exec { prompt })
                } else {
                    Some(BrowserTuiAction::Login)
                }
            }
            AppCommand::Shutdown => Some(BrowserTuiAction::Exit { exit_code: 0 }),
            _ => None,
        }
    }

    fn render(
        &mut self,
        options: &BrowserTuiSessionOptions,
    ) -> Result<BrowserTuiRenderedFrame, String> {
        let width = options
            .terminal_width
            .unwrap_or(DEFAULT_WIDTH)
            .clamp(40, 240);
        let height = options
            .terminal_height
            .unwrap_or(DEFAULT_HEIGHT)
            .clamp(12, 80);
        render_real_browser_tui_session(self, width, height)
    }
}

#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
fn prompt_from_user_inputs(items: &[UserInput]) -> String {
    items
        .iter()
        .filter_map(|item| match item {
            UserInput::Text { text, .. } => Some(text.clone()),
            UserInput::Image { url, .. } => Some(format!("[image: {url}]")),
            UserInput::LocalImage { path, .. } => {
                Some(format!("[local image: {}]", path.display()))
            }
            UserInput::Skill { name, path } => Some(format!("[skill: {name} {}]", path.display())),
            UserInput::Mention { name, path } => Some(format!("[mention: {name} {path}]")),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn new_browser_composer() -> (ChatComposer, UnboundedReceiver<AppEvent>) {
    let (tx, rx) = unbounded_channel();
    let sender = AppEventSender::new(tx);
    let mut composer = ChatComposer::new(
        true,
        sender,
        true,
        BROWSER_TUI_PLACEHOLDER.to_string(),
        true,
    );
    composer.set_collaboration_modes_enabled(true);
    composer.set_goal_command_enabled(true);
    (composer, rx)
}

fn normalize_paste(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn new_user_history_cell(message: String) -> UserHistoryCell {
    UserHistoryCell {
        message,
        text_elements: Vec::new(),
        local_image_paths: Vec::new(),
        remote_image_urls: Vec::new(),
    }
}

fn browser_key_event_to_crossterm(key: BrowserTuiKeyEvent) -> Option<KeyEvent> {
    let code = match key.code {
        BrowserTuiKeyCode::Char(text) => KeyCode::Char(text.chars().next()?),
        BrowserTuiKeyCode::Enter => KeyCode::Enter,
        BrowserTuiKeyCode::Backspace => KeyCode::Backspace,
        BrowserTuiKeyCode::Delete => KeyCode::Delete,
        BrowserTuiKeyCode::Escape => KeyCode::Esc,
        BrowserTuiKeyCode::Tab => KeyCode::Tab,
        BrowserTuiKeyCode::Left => KeyCode::Left,
        BrowserTuiKeyCode::Right => KeyCode::Right,
        BrowserTuiKeyCode::Up => KeyCode::Up,
        BrowserTuiKeyCode::Down => KeyCode::Down,
        BrowserTuiKeyCode::Home => KeyCode::Home,
        BrowserTuiKeyCode::End => KeyCode::End,
        BrowserTuiKeyCode::Unknown(_) => return None,
    };
    let mut modifiers = KeyModifiers::empty();
    if key.ctrl {
        modifiers.insert(KeyModifiers::CONTROL);
    }
    if key.alt {
        modifiers.insert(KeyModifiers::ALT);
    }
    if key.shift {
        modifiers.insert(KeyModifiers::SHIFT);
    }
    Some(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    })
}

enum BrowserTuiSlashSubmission {
    Exit {
        exit_code: i32,
    },
    Clear,
    Message(String),
    Exec {
        command: &'static str,
        prompt: String,
    },
}

fn browser_tui_slash_submission(
    text: &str,
    model: &str,
    directory: &str,
) -> Option<BrowserTuiSlashSubmission> {
    let text = text.trim();
    if !text.starts_with('/') {
        return None;
    }

    let Some((name, args, _)) = parse_slash_name(text) else {
        return Some(BrowserTuiSlashSubmission::Message(
            browser_tui_slash_command_help(),
        ));
    };

    let Ok(command) = SlashCommand::from_str(name) else {
        return Some(BrowserTuiSlashSubmission::Message(format!(
            "Unknown Codex slash command '/{name}'. Type '/' to see available commands."
        )));
    };

    match command {
        SlashCommand::Exit | SlashCommand::Quit => {
            Some(BrowserTuiSlashSubmission::Exit { exit_code: 0 })
        }
        SlashCommand::Clear => Some(BrowserTuiSlashSubmission::Clear),
        SlashCommand::Init => Some(BrowserTuiSlashSubmission::Exec {
            command: command.command(),
            prompt: INIT_PROMPT.to_string(),
        }),
        SlashCommand::Status => Some(BrowserTuiSlashSubmission::Message(format!(
            "Codex browser session status\nmodel: {model}\ndirectory: {directory}\nrenderer: codex_tui history cells on the wasm crossterm adapter\ncommands: !<command> runs through the almostnode shell bridge"
        ))),
        SlashCommand::Model => {
            let message = if args.is_empty() {
                "The native /model picker is recognized, but the browser TUI picker is not wired yet. Set CODEX_MODEL or run codex exec -m <model> for this browser session.".to_string()
            } else {
                format!(
                    "The native /model command is recognized, but changing to '{args}' still needs the browser app-server settings bridge."
                )
            };
            Some(BrowserTuiSlashSubmission::Message(message))
        }
        SlashCommand::Plan => Some(BrowserTuiSlashSubmission::Message(
            "The native /plan command is recognized, but collaboration-mode switching still needs the browser app-server settings bridge.".to_string(),
        )),
        SlashCommand::Feedback => Some(BrowserTuiSlashSubmission::Message(
            "The native /feedback command is recognized, but feedback upload is not available in the sandboxed browser TUI yet.".to_string(),
        )),
        other => Some(BrowserTuiSlashSubmission::Message(format!(
            "The native /{} command is recognized by the forked Codex TUI parser, but its dispatch still depends on the upstream ChatWidget/app loop that is not wired to WASM yet.",
            other.command()
        ))),
    }
}

fn browser_tui_slash_command_help() -> String {
    let commands = built_in_slash_commands()
        .into_iter()
        .map(|(name, _)| format!("/{name}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("Available Codex slash commands: {commands}")
}

fn env_value<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a str> {
    env.iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.as_str())
}

fn browser_tui_width(options: &BrowserTuiSessionOptions) -> u16 {
    options
        .terminal_width
        .unwrap_or(DEFAULT_WIDTH)
        .clamp(40, 240)
}

fn take_scrollback_ansi(lines: &mut Vec<Line<'static>>, width: u16) -> Option<String> {
    if lines.is_empty() {
        return None;
    }
    lines_to_scrollback_ansi(std::mem::take(lines), width)
        .ok()
        .filter(|ansi| !ansi.is_empty())
}

fn lines_to_scrollback_ansi(lines: Vec<Line<'static>>, width: u16) -> Result<String, String> {
    if lines.is_empty() {
        return Ok(String::new());
    }

    let paragraph = Paragraph::new(Text::from(lines.clone())).wrap(Wrap { trim: false });
    let height = paragraph
        .line_count(width)
        .max(1)
        .min(usize::from(u16::MAX)) as u16;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).map_err(|err| err.to_string())?;
    terminal
        .draw(|f| {
            Paragraph::new(Text::from(lines))
                .wrap(Wrap { trim: false })
                .render(f.area(), f.buffer_mut());
        })
        .map_err(|err| err.to_string())?;

    let mut ansi = buffer_rows_to_ansi(terminal.backend().buffer());
    if !ansi.is_empty() {
        ansi.push_str("\r\n");
    }
    Ok(ansi)
}

pub fn render_browser_tui_frame_to_ansi(frame: &BrowserTuiFrame) -> Result<String, String> {
    let backend = TestBackend::new(frame.width, frame.height);
    let mut terminal = Terminal::new(backend).map_err(|err| err.to_string())?;
    terminal
        .draw(|f| {
            let area = f.area();
            f.render_widget(Clear, area);
            render_codex_frame(frame, area, f.buffer_mut());
        })
        .map_err(|err| err.to_string())?;
    Ok(buffer_to_ansi(terminal.backend().buffer()))
}

struct BrowserTuiSessionRender<'a> {
    width: u16,
    height: u16,
    version: Option<String>,
    model: String,
    directory: String,
    status: BrowserTuiStatus,
    history: &'a [Box<dyn HistoryCell>],
    composer: &'a ChatComposer,
}

struct BrowserTuiRenderedFrame {
    ansi: String,
    cursor: Option<BrowserTuiCursorPosition>,
}

fn render_browser_tui_session(
    render: BrowserTuiSessionRender<'_>,
) -> Result<BrowserTuiRenderedFrame, String> {
    let backend = TestBackend::new(render.width, render.height);
    let mut terminal = Terminal::new(backend).map_err(|err| err.to_string())?;
    let mut cursor = None;
    terminal
        .draw(|f| {
            let area = f.area();
            f.render_widget(Clear, area);
            cursor = render_codex_session(&render, area, f.buffer_mut());
        })
        .map_err(|err| err.to_string())?;
    Ok(BrowserTuiRenderedFrame {
        ansi: buffer_to_ansi(terminal.backend().buffer()),
        cursor,
    })
}

#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
fn render_real_browser_tui_session(
    session: &mut RealBrowserInteractiveTuiSession,
    width: u16,
    height: u16,
) -> Result<BrowserTuiRenderedFrame, String> {
    session.chat.pre_draw_tick();
    session.chat.on_terminal_resize(width);
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).map_err(|err| err.to_string())?;
    let mut cursor = None;
    terminal
        .draw(|f| {
            let area = f.area();
            f.render_widget(Clear, area);
            let background = Style::default().bg(Color::Rgb(9, 10, 20));
            Block::default()
                .style(background)
                .render(area, f.buffer_mut());

            let chat_height = session
                .chat
                .desired_height(area.width)
                .clamp(3, area.height);
            let transcript_height = area.height.saturating_sub(chat_height);
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(transcript_height),
                    Constraint::Length(chat_height),
                ])
                .split(area);

            render_real_transcript(&session.transcript_cells, rows[0], f.buffer_mut());
            session.chat.render(rows[1], f.buffer_mut());
            cursor = session
                .chat
                .cursor_pos(rows[1])
                .map(|(x, y)| BrowserTuiCursorPosition { x, y });
        })
        .map_err(|err| err.to_string())?;
    Ok(BrowserTuiRenderedFrame {
        ansi: buffer_to_ansi(terminal.backend().buffer()),
        cursor,
    })
}

#[cfg(all(target_arch = "wasm32", feature = "real-tui-wasm"))]
fn render_real_transcript(cells: &[Arc<dyn HistoryCell>], area: Rect, buf: &mut Buffer) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let mut lines = Vec::new();
    for cell in cells {
        let cell_lines = cell.display_lines(area.width);
        if !cell_lines.is_empty() {
            lines.extend(cell_lines);
        }
    }

    let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
    let scroll = paragraph
        .line_count(area.width)
        .saturating_sub(usize::from(area.height));
    paragraph
        .scroll((u16::try_from(scroll).unwrap_or(u16::MAX), 0))
        .render(area, buf);
}

fn render_codex_frame(frame: &BrowserTuiFrame, area: Rect, buf: &mut Buffer) {
    let background = Style::default().bg(Color::Rgb(9, 10, 20));
    Block::default().style(background).render(area, buf);

    let margin = if area.width > 100 { 2 } else { 1 };
    let content = Rect {
        x: area.x.saturating_add(margin),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(margin * 2),
        height: area.height.saturating_sub(1),
    };
    if content.width == 0 || content.height == 0 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(content);

    render_header(frame, rows[0], buf);
    render_tip(rows[1], buf);
    render_input(frame, rows[2], buf);
    render_transcript(frame, rows[3], buf);
}

fn render_codex_session(
    render: &BrowserTuiSessionRender<'_>,
    area: Rect,
    buf: &mut Buffer,
) -> Option<BrowserTuiCursorPosition> {
    let background = Style::default().bg(Color::Rgb(9, 10, 20));
    Block::default().style(background).render(area, buf);

    let margin = if area.width > 100 { 2 } else { 1 };
    let content = Rect {
        x: area.x.saturating_add(margin),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(margin * 2),
        height: area.height.saturating_sub(1),
    };
    if content.width == 0 || content.height == 0 {
        return None;
    }

    let composer_height = render
        .composer
        .desired_height(content.width)
        .clamp(3, content.height.saturating_sub(3).max(3));
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(composer_height)])
        .split(content);

    render_session_history(render, rows[0], buf);
    render.composer.render(rows[1], buf);
    render
        .composer
        .cursor_pos(rows[1])
        .map(|(x, y)| BrowserTuiCursorPosition { x, y })
}

fn render_session_history(render: &BrowserTuiSessionRender<'_>, area: Rect, buf: &mut Buffer) {
    if area.height == 0 {
        return;
    }

    let mut lines = Vec::new();
    let header = new_session_header_history_cell(render);
    lines.extend(header.display_lines(area.width));
    lines.push(Line::from(""));
    lines.extend(session_tip_lines());

    for cell in render.history {
        let cell_lines = cell.display_lines(area.width);
        if !cell_lines.is_empty() {
            lines.extend(cell_lines);
        }
    }

    if render.status == BrowserTuiStatus::Thinking {
        lines.push(Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Thinking",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
    let scroll = paragraph
        .line_count(area.width)
        .saturating_sub(usize::from(area.height));
    paragraph
        .scroll((u16::try_from(scroll).unwrap_or(u16::MAX), 0))
        .render(area, buf);
}

fn session_tip_lines() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled(
                "Tip:",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Use "),
            Span::styled("/feedback", Style::default().fg(Color::White)),
            Span::raw(" to send logs to the maintainers when something looks off."),
        ]),
        Line::from(""),
    ]
}

#[cfg(all(target_arch = "wasm32", not(feature = "real-tui-wasm")))]
fn new_session_header_history_cell(
    render: &BrowserTuiSessionRender<'_>,
) -> SessionHeaderHistoryCell {
    SessionHeaderHistoryCell::new(
        render.model.clone(),
        std::path::PathBuf::from(render.directory.clone()),
        render.version.clone(),
    )
}

#[cfg(any(
    not(target_arch = "wasm32"),
    all(target_arch = "wasm32", feature = "real-tui-wasm")
))]
fn new_session_header_history_cell(
    render: &BrowserTuiSessionRender<'_>,
) -> SessionHeaderHistoryCell {
    SessionHeaderHistoryCell::new(
        render.model.clone(),
        None,
        false,
        std::path::PathBuf::from(render.directory.clone()),
        env!("CARGO_PKG_VERSION"),
    )
}

fn render_header(frame: &BrowserTuiFrame, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(MAX_HEADER_WIDTH);
    if width < 20 || area.height < 6 {
        return;
    }

    let header_area = Rect {
        width,
        height: 6,
        ..area
    };

    let dim = Style::default().fg(Color::DarkGray);
    let cyan = Style::default().fg(Color::Cyan);
    let model = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let title = vec![
        Span::styled(">_ ", dim),
        Span::styled(
            "OpenAI Codex",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    let title = if let Some(version) = frame.version.as_deref() {
        title
            .into_iter()
            .chain([Span::styled(format!(" (v{version})"), dim)])
            .collect()
    } else {
        title
    };

    let text = vec![
        Line::from(title),
        Line::from(""),
        Line::from(vec![
            Span::styled("model:     ", dim),
            Span::styled(frame.model.clone(), model),
            Span::raw("   "),
            Span::styled("/model", cyan),
            Span::styled(" to change", dim),
        ]),
        Line::from(vec![
            Span::styled("directory: ", dim),
            Span::styled(frame.directory.clone(), Style::default().fg(Color::White)),
        ]),
    ];

    Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(dim)
                .style(Style::default().fg(Color::White)),
        )
        .render(header_area, buf);
}

fn render_tip(area: Rect, buf: &mut Buffer) {
    let text = Line::from(vec![
        Span::styled(
            "Tip:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Use "),
        Span::styled("/feedback", Style::default().fg(Color::White)),
        Span::raw(" to send logs to the maintainers when something looks off."),
    ]);
    Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .render(area, buf);
}

fn render_input(frame: &BrowserTuiFrame, area: Rect, buf: &mut Buffer) {
    if area.height == 0 {
        return;
    }
    let input_area = Rect {
        height: 1,
        y: area.y.saturating_add(1).min(area.y + area.height - 1),
        ..area
    };
    Block::default()
        .style(Style::default().bg(Color::Rgb(39, 41, 53)))
        .render(input_area, buf);

    let input = frame
        .input
        .is_empty()
        .then(|| frame.placeholder.as_deref().unwrap_or_default())
        .unwrap_or(frame.input.as_str());
    let prompt = if frame.status == BrowserTuiStatus::Thinking {
        "• "
    } else {
        "› "
    };
    Paragraph::new(Line::from(vec![
        Span::styled(prompt, Style::default().fg(Color::DarkGray)),
        Span::styled(
            input.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .style(Style::default().bg(Color::Rgb(39, 41, 53)))
    .render(input_area, buf);
}

fn render_transcript(frame: &BrowserTuiFrame, area: Rect, buf: &mut Buffer) {
    if area.height == 0 {
        return;
    }

    let mut lines = Vec::new();
    if frame.status == BrowserTuiStatus::Thinking {
        lines.push(Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Thinking",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
    }

    for item in &frame.transcript {
        push_transcript_entry(&mut lines, item);
    }

    Paragraph::new(lines)
        .style(Style::default().fg(Color::Gray))
        .render(area, buf);
}

fn push_transcript_entry(lines: &mut Vec<Line<'static>>, item: &str) {
    let mut split = item.lines();
    let Some(first) = split.next() else {
        lines.push(Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::styled("", Style::default().fg(Color::Gray)),
        ]));
        return;
    };

    lines.push(Line::from(vec![
        Span::styled("• ", Style::default().fg(Color::DarkGray)),
        Span::styled(first.to_string(), Style::default().fg(Color::Gray)),
    ]));
    for line in split {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(line.to_string(), Style::default().fg(Color::Gray)),
        ]));
    }
}

fn buffer_to_ansi(buffer: &Buffer) -> String {
    let mut out = String::from("\u{1b}[?25l\u{1b}[2J\u{1b}[H");
    push_buffer_rows_ansi(buffer, &mut out);
    out.push_str("\u{1b}[0m");
    out
}

fn buffer_rows_to_ansi(buffer: &Buffer) -> String {
    let mut out = String::new();
    push_buffer_rows_ansi(buffer, &mut out);
    out.push_str("\u{1b}[0m");
    out
}

fn push_buffer_rows_ansi(buffer: &Buffer, out: &mut String) {
    let mut current_fg = Color::Reset;
    let mut current_bg = Color::Reset;
    let mut current_modifier = Modifier::empty();

    for y in 0..buffer.area.height {
        if y > 0 {
            out.push_str("\u{1b}[0m\r\n");
            current_fg = Color::Reset;
            current_bg = Color::Reset;
            current_modifier = Modifier::empty();
        }

        let mut line = String::new();
        for x in 0..buffer.area.width {
            let Some(cell) = buffer.cell((x, y)) else {
                continue;
            };
            push_style_diff(
                &mut line,
                &mut current_fg,
                &mut current_bg,
                &mut current_modifier,
                cell.fg,
                cell.bg,
                cell.modifier,
            );
            line.push_str(cell.symbol());
        }
        out.push_str(line.trim_end());
    }
}

fn push_style_diff(
    out: &mut String,
    current_fg: &mut Color,
    current_bg: &mut Color,
    current_modifier: &mut Modifier,
    fg: Color,
    bg: Color,
    modifier: Modifier,
) {
    if *current_fg == fg && *current_bg == bg && *current_modifier == modifier {
        return;
    }

    out.push_str("\u{1b}[0m");
    push_fg(out, fg);
    push_bg(out, bg);
    if modifier.contains(Modifier::BOLD) {
        out.push_str("\u{1b}[1m");
    }
    if modifier.contains(Modifier::DIM) {
        out.push_str("\u{1b}[2m");
    }
    if modifier.contains(Modifier::ITALIC) {
        out.push_str("\u{1b}[3m");
    }
    if modifier.contains(Modifier::CROSSED_OUT) {
        out.push_str("\u{1b}[9m");
    }

    *current_fg = fg;
    *current_bg = bg;
    *current_modifier = modifier;
}

fn push_fg(out: &mut String, color: Color) {
    match color {
        Color::Reset => {}
        Color::Black => out.push_str("\u{1b}[30m"),
        Color::Red => out.push_str("\u{1b}[31m"),
        Color::Green => out.push_str("\u{1b}[32m"),
        Color::Yellow => out.push_str("\u{1b}[33m"),
        Color::Blue => out.push_str("\u{1b}[34m"),
        Color::Magenta => out.push_str("\u{1b}[35m"),
        Color::Cyan => out.push_str("\u{1b}[36m"),
        Color::Gray | Color::White => out.push_str("\u{1b}[37m"),
        Color::DarkGray => out.push_str("\u{1b}[90m"),
        Color::LightRed => out.push_str("\u{1b}[91m"),
        Color::LightGreen => out.push_str("\u{1b}[92m"),
        Color::LightYellow => out.push_str("\u{1b}[93m"),
        Color::LightBlue => out.push_str("\u{1b}[94m"),
        Color::LightMagenta => out.push_str("\u{1b}[95m"),
        Color::LightCyan => out.push_str("\u{1b}[96m"),
        Color::Rgb(r, g, b) => out.push_str(&format!("\u{1b}[38;2;{r};{g};{b}m")),
        Color::Indexed(index) => out.push_str(&format!("\u{1b}[38;5;{index}m")),
    }
}

fn push_bg(out: &mut String, color: Color) {
    match color {
        Color::Reset => {}
        Color::Black => out.push_str("\u{1b}[40m"),
        Color::Red => out.push_str("\u{1b}[41m"),
        Color::Green => out.push_str("\u{1b}[42m"),
        Color::Yellow => out.push_str("\u{1b}[43m"),
        Color::Blue => out.push_str("\u{1b}[44m"),
        Color::Magenta => out.push_str("\u{1b}[45m"),
        Color::Cyan => out.push_str("\u{1b}[46m"),
        Color::Gray | Color::White => out.push_str("\u{1b}[47m"),
        Color::DarkGray => out.push_str("\u{1b}[100m"),
        Color::LightRed => out.push_str("\u{1b}[101m"),
        Color::LightGreen => out.push_str("\u{1b}[102m"),
        Color::LightYellow => out.push_str("\u{1b}[103m"),
        Color::LightBlue => out.push_str("\u{1b}[104m"),
        Color::LightMagenta => out.push_str("\u{1b}[105m"),
        Color::LightCyan => out.push_str("\u{1b}[106m"),
        Color::Rgb(r, g, b) => out.push_str(&format!("\u{1b}[48;2;{r};{g};{b}m")),
        Color::Indexed(index) => out.push_str(&format!("\u{1b}[48;5;{index}m")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_session_start_renders_codex_frame() {
        let mut session = BrowserInteractiveTuiSession::default();
        let result = session
            .start(
                None,
                &BrowserTuiSessionOptions {
                    cwd: Some("/workspace".to_string()),
                    terminal_width: Some(80),
                    terminal_height: Some(24),
                    ..BrowserTuiSessionOptions::default()
                },
            )
            .expect("frame render");

        assert!(result.ansi.contains("OpenAI Codex"));
        assert!(result.ansi.contains("/workspace"));
        assert_eq!(result.action, BrowserTuiAction::Login);
    }

    #[test]
    fn browser_session_submit_classifies_shell_command() {
        let mut session = BrowserInteractiveTuiSession::default();
        session
            .start(None, &BrowserTuiSessionOptions::default())
            .expect("start frame");
        session
            .set_input("!ls -la".to_string(), &BrowserTuiSessionOptions::default())
            .expect("input frame");

        let result = session
            .submit(&BrowserTuiSessionOptions::default())
            .expect("submit frame");

        assert_eq!(
            result.action,
            BrowserTuiAction::Shell {
                command: "ls -la".to_string(),
            }
        );
        assert!(result.ansi.contains("Thinking"));
    }

    #[test]
    fn browser_session_prompts_for_auth_instead_of_exec_without_credentials() {
        let mut session = BrowserInteractiveTuiSession::default();
        session
            .start(None, &BrowserTuiSessionOptions::default())
            .expect("start frame");
        session
            .set_input("hey".to_string(), &BrowserTuiSessionOptions::default())
            .expect("input frame");

        let result = session
            .submit(&BrowserTuiSessionOptions::default())
            .expect("submit frame");

        assert_eq!(result.action, BrowserTuiAction::Login);
        assert!(!result.ansi.contains("Sign in to Codex"));
        assert!(!result.ansi.contains("codex exec exited with code 1"));
    }

    #[test]
    fn browser_session_execs_with_credentials() {
        let mut session = BrowserInteractiveTuiSession::default();
        let options = BrowserTuiSessionOptions {
            env: vec![("OPENAI_API_KEY".to_string(), "test-key".to_string())],
            ..BrowserTuiSessionOptions::default()
        };
        session.start(None, &options).expect("start frame");
        session
            .set_input("hey".to_string(), &options)
            .expect("input frame");

        let result = session.submit(&options).expect("submit frame");

        assert_eq!(
            result.action,
            BrowserTuiAction::Exec {
                prompt: "hey".to_string(),
            }
        );
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn browser_session_header_uses_release_version_env() {
        let mut session = BrowserInteractiveTuiSession::default();
        let result = session
            .start(
                None,
                &BrowserTuiSessionOptions {
                    env: vec![("CODEX_CLI_VERSION".to_string(), "0.137.0".to_string())],
                    ..BrowserTuiSessionOptions::default()
                },
            )
            .expect("start frame");

        assert!(result.ansi.contains("OpenAI Codex"));
        assert!(result.ansi.contains("v0.137.0"));
    }

    #[test]
    fn browser_session_key_events_classify_shell_command() {
        let mut session = BrowserInteractiveTuiSession::default();
        session
            .start(None, &BrowserTuiSessionOptions::default())
            .expect("start frame");

        for text in ["!", "l", "s"] {
            session
                .handle_event(
                    BrowserTuiEvent::Key(BrowserTuiKeyEvent {
                        code: BrowserTuiKeyCode::Char(text.to_string()),
                        ctrl: false,
                        alt: false,
                        shift: false,
                    }),
                    &BrowserTuiSessionOptions::default(),
                )
                .expect("key frame");
        }

        let result = session
            .handle_event(
                BrowserTuiEvent::Key(BrowserTuiKeyEvent {
                    code: BrowserTuiKeyCode::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }),
                &BrowserTuiSessionOptions::default(),
            )
            .expect("enter frame");

        assert_eq!(
            result.action,
            BrowserTuiAction::Shell {
                command: "ls".to_string(),
            }
        );
    }

    #[test]
    fn browser_session_paste_and_backspace_update_input() {
        let mut session = BrowserInteractiveTuiSession::default();
        session
            .start(None, &BrowserTuiSessionOptions::default())
            .expect("start frame");

        let result = session
            .handle_event(
                BrowserTuiEvent::Paste("hello\r\nbrowser".to_string()),
                &BrowserTuiSessionOptions::default(),
            )
            .expect("paste frame");
        assert!(result.ansi.contains("hello"));
        assert!(result.ansi.contains("browser"));

        let result = session
            .handle_event(
                BrowserTuiEvent::Key(BrowserTuiKeyEvent {
                    code: BrowserTuiKeyCode::Backspace,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }),
                &BrowserTuiSessionOptions::default(),
            )
            .expect("backspace frame");

        assert!(result.ansi.contains("browse"));
    }

    #[test]
    fn browser_session_append_result_updates_transcript() {
        let mut session = BrowserInteractiveTuiSession::default();
        session
            .start(None, &BrowserTuiSessionOptions::default())
            .expect("start frame");
        session
            .set_input("!ls".to_string(), &BrowserTuiSessionOptions::default())
            .expect("input frame");
        session
            .submit(&BrowserTuiSessionOptions::default())
            .expect("submit frame");

        let result = session
            .append_result(
                BrowserTuiAppendResult {
                    kind: BrowserTuiResultKind::Shell,
                    stdout: "README.md\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                &BrowserTuiSessionOptions::default(),
            )
            .expect("result frame");

        assert_eq!(result.action, BrowserTuiAction::None);
        assert!(result.ansi.contains("README.md"));
    }

    #[test]
    fn browser_session_append_exec_result_uses_agent_markdown_cell() {
        let mut session = BrowserInteractiveTuiSession::default();
        session
            .start(None, &BrowserTuiSessionOptions::default())
            .expect("start frame");
        session
            .set_input(
                "Explain markdown rendering".to_string(),
                &BrowserTuiSessionOptions::default(),
            )
            .expect("input frame");
        session
            .submit(&BrowserTuiSessionOptions::default())
            .expect("submit frame");

        let result = session
            .append_result(
                BrowserTuiAppendResult {
                    kind: BrowserTuiResultKind::Exec,
                    stdout: "**Native assistant message**\n\n- rendered by Codex markdown\n"
                        .to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                &BrowserTuiSessionOptions::default(),
            )
            .expect("result frame");

        assert_eq!(result.action, BrowserTuiAction::None);
        assert!(result.ansi.contains("Native assistant message"));
        assert!(result.ansi.contains("rendered by Codex markdown"));
    }

    #[test]
    fn browser_session_append_plan_update_uses_history_cell() {
        let mut session = BrowserInteractiveTuiSession::default();
        session
            .start(None, &BrowserTuiSessionOptions::default())
            .expect("start frame");

        let result = session
            .append_plan_update(
                codex_protocol::plan_tool::UpdatePlanArgs {
                    explanation: Some("Browser wasm plan update".to_string()),
                    plan: vec![
                        codex_protocol::plan_tool::PlanItemArg {
                            step: "Compile the forked Codex TUI renderer for wasm".to_string(),
                            status: codex_protocol::plan_tool::StepStatus::Completed,
                        },
                        codex_protocol::plan_tool::PlanItemArg {
                            step: "Route browser terminal input through the wasm frame loop"
                                .to_string(),
                            status: codex_protocol::plan_tool::StepStatus::InProgress,
                        },
                    ],
                },
                &BrowserTuiSessionOptions::default(),
            )
            .expect("plan frame");

        assert_eq!(result.action, BrowserTuiAction::None);
        assert!(result.ansi.contains("Updated Plan"));
        assert!(
            result
                .ansi
                .contains("Compile the forked Codex TUI renderer for wasm")
        );
        assert!(
            result
                .ansi
                .contains("Route browser terminal input through the wasm frame loop")
        );
    }
}
