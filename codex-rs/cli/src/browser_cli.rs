use clap::Parser;
use clap::Subcommand;
use codex_protocol::plan_tool::PlanItemArg;
use codex_protocol::plan_tool::StepStatus;
use codex_protocol::plan_tool::UpdatePlanArgs;
use codex_tui::browser::BrowserInteractiveTuiSession;
pub use codex_tui::browser::BrowserTuiAction;
use codex_tui::browser::BrowserTuiAppendResult;
use codex_tui::browser::BrowserTuiEvent;
use codex_tui::browser::BrowserTuiKeyCode;
use codex_tui::browser::BrowserTuiKeyEvent;
use codex_tui::browser::BrowserTuiResultKind;
pub use codex_tui::browser::BrowserTuiRunResult;
use codex_tui::browser::BrowserTuiSessionOptions;

#[derive(Debug, Default)]
pub struct BrowserCodexCliSession {
    auth_source: Option<String>,
    auth_env: Vec<(String, String)>,
    tui: BrowserInteractiveTuiSession,
}

#[derive(Debug, Default)]
pub struct BrowserRunOptions {
    pub cwd: Option<String>,
    pub stdin: Option<String>,
    pub env: Vec<(String, String)>,
    pub terminal_width: Option<u16>,
    pub terminal_height: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserRunResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub env: Vec<(String, String)>,
    pub browser_exec: Option<BrowserExecPlan>,
    pub browser_login: Option<BrowserLoginRequest>,
    pub browser_tui: Option<BrowserTuiRunResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserLoginRequest {
    pub method: BrowserLoginMethod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserLoginMethod {
    ChatGpt,
    DeviceCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserExecPlan {
    pub prompt: String,
    pub model: String,
    pub instructions: String,
    pub tool_choice: String,
    pub parallel_tool_calls: bool,
    pub store: bool,
    pub stream: bool,
    pub json: bool,
    pub output_last_message_path: Option<String>,
    pub warnings: Vec<String>,
    pub cwd: Option<String>,
    pub apply_patch_grammar: Option<String>,
}

#[derive(Debug, Parser)]
#[command(
    name = "codex",
    bin_name = "codex",
    version,
    about = "Codex CLI",
    subcommand_negates_reqs = true,
    override_usage = "codex [OPTIONS] [PROMPT]\n       codex [OPTIONS] <COMMAND> [ARGS]"
)]
struct BrowserCodexCli {
    #[arg(short = 'C', long = "cd", value_name = "DIR", global = true)]
    cwd: Option<String>,

    #[arg(short = 'c', long = "config", value_name = "KEY=VALUE", action = clap::ArgAction::Append, global = true)]
    config: Vec<String>,

    #[arg(long = "enable", value_name = "FEATURE", action = clap::ArgAction::Append, global = true)]
    enable: Vec<String>,

    #[arg(long = "disable", value_name = "FEATURE", action = clap::ArgAction::Append, global = true)]
    disable: Vec<String>,

    #[arg(long = "strict-config", default_value_t = false, global = true)]
    strict_config: bool,

    #[arg(value_name = "PROMPT")]
    prompt: Option<String>,

    #[command(subcommand)]
    command: Option<BrowserSubcommand>,
}

#[derive(Debug, Subcommand)]
enum BrowserSubcommand {
    /// Run Codex non-interactively.
    #[command(visible_alias = "e")]
    Exec(TrailingArgs),

    /// Run a code review non-interactively.
    Review(TrailingArgs),

    /// Manage login.
    Login(LoginCommand),

    /// Remove stored authentication credentials.
    Logout,

    /// Manage external MCP servers for Codex.
    Mcp(TrailingArgs),

    /// Manage Codex plugins.
    Plugin(TrailingArgs),

    /// Start Codex as an MCP server (stdio).
    McpServer(TrailingArgs),

    /// [experimental] Run the app server or related tooling.
    AppServer(TrailingArgs),

    /// [experimental] Manage the app-server daemon with remote control enabled.
    RemoteControl(TrailingArgs),

    /// Generate shell completion scripts.
    Completion(CompletionCommand),

    /// Update Codex to the latest version.
    Update,

    /// Diagnose local Codex installation, config, auth, and runtime health.
    Doctor,

    /// Run commands within a Codex-provided sandbox.
    Sandbox(TrailingArgs),

    /// Debugging tools.
    Debug(TrailingArgs),

    /// Execpolicy tooling.
    #[command(hide = true)]
    Execpolicy(TrailingArgs),

    /// Apply the latest diff produced by Codex agent as a git apply to your local working tree.
    #[command(visible_alias = "a")]
    Apply(TrailingArgs),

    /// Resume a previous interactive session.
    Resume(SessionCommand),

    /// Archive a saved session by id or session name.
    Archive(SessionTargetCommand),

    /// Unarchive a saved session by id or session name.
    Unarchive(SessionTargetCommand),

    /// Fork a previous interactive session.
    Fork(SessionCommand),

    /// [EXPERIMENTAL] Browse tasks from Codex Cloud and apply changes locally.
    #[command(name = "cloud", alias = "cloud-tasks")]
    Cloud(TrailingArgs),

    /// Internal: run the responses API proxy.
    #[command(hide = true)]
    ResponsesApiProxy(TrailingArgs),

    /// Internal: relay stdio to a Unix domain socket.
    #[command(hide = true, name = "stdio-to-uds")]
    StdioToUds(TrailingArgs),

    /// [EXPERIMENTAL] Run the standalone exec-server service.
    ExecServer(TrailingArgs),

    /// Inspect feature flags.
    Features(FeaturesCommand),
}

#[derive(Debug, Parser)]
struct TrailingArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(Debug, Parser)]
struct LoginCommand {
    #[arg(long = "with-api-key")]
    with_api_key: bool,

    #[arg(long = "with-access-token")]
    with_access_token: bool,

    #[arg(long = "device-auth")]
    device_auth: bool,

    #[command(subcommand)]
    command: Option<LoginSubcommand>,
}

#[derive(Debug, Subcommand)]
enum LoginSubcommand {
    /// Show login status.
    Status,
}

#[derive(Debug, Parser)]
struct CompletionCommand {
    #[arg(value_name = "SHELL", default_value = "bash")]
    shell: String,
}

#[derive(Debug, Parser)]
struct SessionCommand {
    #[arg(value_name = "SESSION_ID")]
    session_id: Option<String>,

    #[arg(long = "last", default_value_t = false)]
    last: bool,

    #[arg(long = "all", default_value_t = false)]
    all: bool,
}

#[derive(Debug, Parser)]
struct SessionTargetCommand {
    #[arg(value_name = "SESSION")]
    target: String,
}

#[derive(Debug, Parser)]
struct FeaturesCommand {
    #[command(subcommand)]
    command: FeaturesSubcommand,
}

#[derive(Debug, Subcommand)]
enum FeaturesSubcommand {
    /// List known features with their browser support state.
    List,
    /// Enable a feature in browser session state.
    Enable(FeatureName),
    /// Disable a feature in browser session state.
    Disable(FeatureName),
}

#[derive(Debug, Parser)]
struct FeatureName {
    feature: String,
}

impl BrowserCodexCliSession {
    pub fn new() -> BrowserCodexCliSession {
        BrowserCodexCliSession::default()
    }

    pub fn run(&mut self, args: Vec<String>, options: BrowserRunOptions) -> BrowserRunResult {
        if matches!(args.as_slice(), [arg] if arg == "--version" || arg == "-V") {
            return BrowserRunResult::stdout(
                format!("codex {}\n", browser_cli_version(&options)),
                0,
            );
        }

        let options = self.options_with_session_auth(options);

        let argv = std::iter::once(String::from("codex")).chain(args);
        let cli = match BrowserCodexCli::try_parse_from(argv) {
            Ok(cli) => cli,
            Err(error) => {
                let output = error.to_string();
                return if error.use_stderr() {
                    BrowserRunResult::stderr(output, 2)
                } else {
                    BrowserRunResult::stdout(output, 0)
                };
            }
        };

        self.run_command(cli, options)
    }

    fn options_with_session_auth(&self, mut options: BrowserRunOptions) -> BrowserRunOptions {
        for (name, value) in &self.auth_env {
            if env_value(&options.env, name).is_none() {
                options.env.push((name.clone(), value.clone()));
            }
        }
        options
    }

    fn run_command(
        &mut self,
        cli: BrowserCodexCli,
        options: BrowserRunOptions,
    ) -> BrowserRunResult {
        match cli.command {
            None => self.run_interactive(cli.prompt, options),
            Some(BrowserSubcommand::Login(command)) => self.run_login(command, options),
            Some(BrowserSubcommand::Logout) => self.run_logout(),
            Some(BrowserSubcommand::Completion(command)) => {
                BrowserRunResult::stdout(render_completion_placeholder(command.shell), 0)
            }
            Some(BrowserSubcommand::Features(command)) => run_features(command),
            Some(BrowserSubcommand::Doctor) => run_doctor(options),
            Some(BrowserSubcommand::Exec(args)) => run_exec(cli.cwd, cli.config, args, options),
            Some(BrowserSubcommand::Review(args)) => unsupported("review", args.args),
            Some(BrowserSubcommand::Mcp(args)) => unsupported("mcp", args.args),
            Some(BrowserSubcommand::Plugin(args)) => unsupported("plugin", args.args),
            Some(BrowserSubcommand::McpServer(args)) => unsupported("mcp-server", args.args),
            Some(BrowserSubcommand::AppServer(args)) => unsupported("app-server", args.args),
            Some(BrowserSubcommand::RemoteControl(args)) => {
                unsupported("remote-control", args.args)
            }
            Some(BrowserSubcommand::Update) => unsupported("update", Vec::new()),
            Some(BrowserSubcommand::Sandbox(args)) => unsupported("sandbox", args.args),
            Some(BrowserSubcommand::Debug(args)) => self.run_debug(args, options),
            Some(BrowserSubcommand::Execpolicy(args)) => unsupported("execpolicy", args.args),
            Some(BrowserSubcommand::Apply(args)) => unsupported("apply", args.args),
            Some(BrowserSubcommand::Resume(command)) => unsupported(
                "resume",
                session_args(command.session_id, command.last, command.all),
            ),
            Some(BrowserSubcommand::Archive(command)) => {
                unsupported("archive", vec![command.target])
            }
            Some(BrowserSubcommand::Unarchive(command)) => {
                unsupported("unarchive", vec![command.target])
            }
            Some(BrowserSubcommand::Fork(command)) => unsupported(
                "fork",
                session_args(command.session_id, command.last, command.all),
            ),
            Some(BrowserSubcommand::Cloud(args)) => unsupported("cloud", args.args),
            Some(BrowserSubcommand::ResponsesApiProxy(args)) => {
                unsupported("responses-api-proxy", args.args)
            }
            Some(BrowserSubcommand::StdioToUds(args)) => unsupported("stdio-to-uds", args.args),
            Some(BrowserSubcommand::ExecServer(args)) => unsupported("exec-server", args.args),
        }
    }

    fn run_interactive(
        &mut self,
        prompt: Option<String>,
        options: BrowserRunOptions,
    ) -> BrowserRunResult {
        browser_tui_result(self.tui.start(prompt, &browser_tui_options(&options)))
    }

    fn run_debug(&mut self, args: TrailingArgs, options: BrowserRunOptions) -> BrowserRunResult {
        let mut args = args.args;
        if args.first().map(String::as_str) == Some("browser-tui-frame") {
            args.remove(0);
            #[cfg(target_arch = "wasm32")]
            {
                return crate::browser_tui::render_browser_tui_frame(args, options);
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                return unsupported(
                    "debug",
                    std::iter::once("browser-tui-frame".to_string())
                        .chain(args)
                        .collect(),
                );
            }
        }

        match args.first().map(String::as_str) {
            Some("browser-tui-start") => {
                args.remove(0);
                match parse_browser_tui_input_arg(args) {
                    Ok(prompt) => {
                        browser_tui_result(self.tui.start(prompt, &browser_tui_options(&options)))
                    }
                    Err(result) => result,
                }
            }
            Some("browser-tui-input") => {
                args.remove(0);
                match parse_browser_tui_required_input_arg(args) {
                    Ok(input) => browser_tui_result(
                        self.tui.set_input(input, &browser_tui_options(&options)),
                    ),
                    Err(result) => result,
                }
            }
            Some("browser-tui-event") => {
                args.remove(0);
                match parse_browser_tui_event_args(args) {
                    Ok(event) => browser_tui_result(
                        self.tui.handle_event(event, &browser_tui_options(&options)),
                    ),
                    Err(result) => result,
                }
            }
            Some("browser-tui-submit") => {
                browser_tui_result(self.tui.submit(&browser_tui_options(&options)))
            }
            Some("browser-tui-result") => {
                args.remove(0);
                match parse_browser_tui_result_args(args) {
                    Ok(result) => browser_tui_result(
                        self.tui
                            .append_result(result, &browser_tui_options(&options)),
                    ),
                    Err(result) => result,
                }
            }
            Some("browser-tui-agent-message") => {
                args.remove(0);
                match parse_browser_tui_agent_message_args(args) {
                    Ok(message) => browser_tui_result(
                        self.tui
                            .append_agent_message(message, &browser_tui_options(&options)),
                    ),
                    Err(result) => result,
                }
            }
            Some("browser-tui-plan-update") => {
                args.remove(0);
                match parse_browser_tui_plan_update_args(args) {
                    Ok(update) => browser_tui_result(
                        self.tui
                            .append_plan_update(update, &browser_tui_options(&options)),
                    ),
                    Err(result) => result,
                }
            }
            Some("browser-tui-notification") => {
                args.remove(0);
                match parse_browser_tui_notification_args(args) {
                    Ok(notification) => browser_tui_result(
                        self.tui.apply_server_notification_json(
                            notification,
                            &browser_tui_options(&options),
                        ),
                    ),
                    Err(result) => result,
                }
            }
            _ => unsupported("debug", args),
        }
    }

    fn run_login(&mut self, command: LoginCommand, options: BrowserRunOptions) -> BrowserRunResult {
        if matches!(command.command, Some(LoginSubcommand::Status)) {
            return self.run_login_status(options);
        }

        if command.with_api_key && command.with_access_token {
            return BrowserRunResult::stderr(
                "Choose one login credential source: --with-api-key or --with-access-token.\n",
                1,
            );
        }

        if command.with_api_key {
            let api_key = options.stdin.as_deref().unwrap_or_default().trim();
            let api_key = if api_key.is_empty() {
                env_value(&options.env, "CODEX_API_KEY")
                    .or_else(|| env_value(&options.env, "OPENAI_API_KEY"))
                    .unwrap_or_default()
                    .trim()
            } else {
                api_key
            };
            if api_key.is_empty() {
                return BrowserRunResult::stderr(
                    "codex login --with-api-key in the browser requires an API key on stdin or OPENAI_API_KEY/CODEX_API_KEY in the browser shell environment.\n",
                    1,
                );
            }

            self.auth_source = Some("stdin api key/browser session".to_string());
            self.auth_env = vec![
                ("OPENAI_API_KEY".to_string(), api_key.to_string()),
                ("CODEX_API_KEY".to_string(), api_key.to_string()),
            ];
            return BrowserRunResult::stdout(
                "Stored OpenAI API key for this browser Codex session.\n",
                0,
            )
            .with_env(self.auth_env.clone());
        }

        if command.with_access_token {
            let access_token = options.stdin.as_deref().unwrap_or_default().trim();
            let access_token = if access_token.is_empty() {
                env_value(&options.env, "CODEX_ACCESS_TOKEN")
                    .unwrap_or_default()
                    .trim()
            } else {
                access_token
            };
            if access_token.is_empty() {
                return BrowserRunResult::stderr(
                    "codex login --with-access-token in the browser requires an access token on stdin or CODEX_ACCESS_TOKEN in the browser shell environment.\n",
                    1,
                );
            }

            self.auth_source = Some("stdin access token/browser session".to_string());
            self.auth_env = vec![("CODEX_ACCESS_TOKEN".to_string(), access_token.to_string())];
            return BrowserRunResult::stdout(
                "Stored Codex access token for this browser Codex session.\n",
                0,
            )
            .with_env(self.auth_env.clone());
        }

        if command.device_auth {
            return BrowserRunResult::browser_login(BrowserLoginRequest {
                method: BrowserLoginMethod::DeviceCode,
            });
        }

        BrowserRunResult::browser_login(BrowserLoginRequest {
            method: BrowserLoginMethod::DeviceCode,
        })
    }

    fn run_login_status(&self, options: BrowserRunOptions) -> BrowserRunResult {
        if let Some(source) = self.auth_source.as_ref() {
            return BrowserRunResult::stdout(
                format!("Logged in for this browser session via {source}.\n"),
                0,
            );
        }

        if env_value(&options.env, "CODEX_ACCESS_TOKEN").is_some() {
            return BrowserRunResult::stdout(
                "Logged in for this browser session via CODEX_ACCESS_TOKEN.\n",
                0,
            );
        }

        if env_value(&options.env, "CODEX_API_KEY").is_some() {
            return BrowserRunResult::stdout(
                "Logged in for this browser session via CODEX_API_KEY.\n",
                0,
            );
        }

        if env_value(&options.env, "OPENAI_API_KEY").is_some() {
            return BrowserRunResult::stdout(
                "Logged in for this browser session via OPENAI_API_KEY.\n",
                0,
            );
        }

        BrowserRunResult::stderr(
            "Not logged in. Set CODEX_API_KEY, OPENAI_API_KEY, or CODEX_ACCESS_TOKEN in the browser shell environment.\n",
            1,
        )
    }

    fn run_logout(&mut self) -> BrowserRunResult {
        self.auth_source = None;
        self.auth_env.clear();
        BrowserRunResult::stdout("Cleared browser Codex session credentials.\n", 0)
    }
}

const DEFAULT_BROWSER_CODEX_MODEL: &str = "gpt-5.5";
const DEFAULT_BROWSER_CODEX_INSTRUCTIONS: &str = include_str!("../../core/gpt-5.2-codex_prompt.md");

fn run_exec(
    cwd: Option<String>,
    config: Vec<String>,
    args: TrailingArgs,
    options: BrowserRunOptions,
) -> BrowserRunResult {
    match parse_exec_plan(args.args, &config, &options, cwd) {
        Ok(plan) => BrowserRunResult::browser_exec(plan),
        Err(result) => result,
    }
}

fn parse_exec_plan(
    args: Vec<String>,
    config: &[String],
    options: &BrowserRunOptions,
    cwd: Option<String>,
) -> Result<BrowserExecPlan, BrowserRunResult> {
    let mut json = false;
    let mut model: Option<String> = None;
    let mut output_last_message_path: Option<String> = None;
    let mut prompt_parts: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];

        if arg == "--" {
            prompt_parts.extend(args[index + 1..].iter().cloned());
            break;
        }

        if arg == "--help" || arg == "-h" {
            return Err(BrowserRunResult::stdout(render_exec_help(), 0));
        }

        if arg == "resume" || arg == "review" {
            return Err(unsupported_exec_mode(arg));
        }

        if matches!(
            arg.as_str(),
            "--skip-git-repo-check"
                | "--ephemeral"
                | "--ignore-user-config"
                | "--ignore-rules"
                | "--strict-config"
        ) {
            index += 1;
            continue;
        }

        if arg == "--full-auto" {
            warnings.push(
                "warning: `--full-auto` is deprecated; use `--sandbox workspace-write` instead."
                    .to_string(),
            );
            index += 1;
            continue;
        }

        if arg == "--json" || arg == "--experimental-json" {
            json = true;
            index += 1;
            continue;
        }

        if arg == "-m" || arg == "--model" {
            let Some(value) = args.get(index + 1) else {
                return Err(missing_exec_flag_value(arg));
            };
            model = Some(value.clone());
            index += 2;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--model=") {
            model = Some(value.to_string());
            index += 1;
            continue;
        }

        if arg.starts_with("-m") && arg.len() > 2 {
            model = Some(arg[2..].to_string());
            index += 1;
            continue;
        }

        if arg == "-o" || arg == "--output-last-message" {
            let Some(value) = args.get(index + 1) else {
                return Err(missing_exec_flag_value(arg));
            };
            output_last_message_path = Some(value.clone());
            index += 2;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--output-last-message=") {
            output_last_message_path = Some(value.to_string());
            index += 1;
            continue;
        }

        if arg.starts_with("-o") && arg.len() > 2 {
            output_last_message_path = Some(arg[2..].to_string());
            index += 1;
            continue;
        }

        if arg == "--color" {
            if args.get(index + 1).is_none() {
                return Err(missing_exec_flag_value(arg));
            }
            index += 2;
            continue;
        }

        if arg.starts_with("--color=") {
            index += 1;
            continue;
        }

        if arg == "--output-schema" || arg.starts_with("--output-schema=") {
            return Err(unsupported_exec_mode("--output-schema"));
        }

        if arg.starts_with('-') {
            return Err(BrowserRunResult::stderr(
                format!(
                    "error: unsupported browser codex exec option '{arg}'\n\n{}",
                    render_exec_usage()
                ),
                2,
            ));
        }

        prompt_parts.push(arg.clone());
        index += 1;
    }

    let Some(prompt) = build_exec_prompt(&prompt_parts, options.stdin.as_deref()) else {
        return Err(BrowserRunResult::stderr(
            format!(
                "error: codex exec requires a prompt or stdin in the browser\n\n{}",
                render_exec_usage()
            ),
            2,
        ));
    };

    Ok(BrowserExecPlan {
        prompt,
        model: model
            .or_else(|| config_model(config))
            .or_else(|| env_value(&options.env, "CODEX_MODEL").map(str::to_string))
            .or_else(|| env_value(&options.env, "OPENAI_MODEL").map(str::to_string))
            .unwrap_or_else(|| DEFAULT_BROWSER_CODEX_MODEL.to_string()),
        instructions: DEFAULT_BROWSER_CODEX_INSTRUCTIONS.to_string(),
        tool_choice: "auto".to_string(),
        parallel_tool_calls: false,
        store: false,
        stream: true,
        json,
        output_last_message_path,
        warnings,
        cwd: cwd.or_else(|| options.cwd.clone()),
        apply_patch_grammar: Some(include_str!("../../core/src/tools/handlers/apply_patch.lark").to_string()),
    })
}

fn build_exec_prompt(parts: &[String], stdin: Option<&str>) -> Option<String> {
    let stdin = stdin.unwrap_or_default();

    if parts.len() == 1 && parts[0] == "-" {
        let stdin = stdin.trim();
        return (!stdin.is_empty()).then(|| stdin.to_string());
    }

    let prompt_from_args = parts.join(" ").trim().to_string();
    let has_stdin = !stdin.trim().is_empty();
    if !prompt_from_args.is_empty() && has_stdin {
        return Some(format!("{prompt_from_args}\n\n<stdin>\n{stdin}\n</stdin>"));
    }
    if !prompt_from_args.is_empty() {
        return Some(prompt_from_args);
    }
    has_stdin.then(|| stdin.to_string())
}

fn config_model(config: &[String]) -> Option<String> {
    for entry in config {
        let Some((key, value)) = entry.split_once('=') else {
            continue;
        };
        if key.trim() != "model" {
            continue;
        }
        return Some(strip_toml_string(value.trim()).to_string());
    }
    None
}

fn strip_toml_string(value: &str) -> &str {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn unsupported_exec_mode(mode: &str) -> BrowserRunResult {
    BrowserRunResult::stderr(
        format!(
            "codex exec {mode} is parsed in the browser, but this mode still depends on native Codex runtime services.\n\
             Supported browser slice: one-shot `codex exec [OPTIONS] [PROMPT]` through the browser host bridge.\n"
        ),
        78,
    )
}

fn missing_exec_flag_value(flag: &str) -> BrowserRunResult {
    BrowserRunResult::stderr(
        format!(
            "error: a value is required for '{flag}'\n\n{}",
            render_exec_usage()
        ),
        2,
    )
}

fn render_exec_help() -> String {
    format!(
        "{}

Run Codex non-interactively in the browser.

Options:
  -m, --model <MODEL>              Model to use. Defaults to {DEFAULT_BROWSER_CODEX_MODEL}.
      --json, --experimental-json  Print browser exec events as JSONL.
  -o, --output-last-message <FILE> Write the final response text to a file.
      --skip-git-repo-check        Accepted for native CLI compatibility.
      --ephemeral                  Accepted for native CLI compatibility.
  -h, --help                       Print help.
",
        render_exec_usage()
    )
}

fn render_exec_usage() -> &'static str {
    "Usage: codex exec [OPTIONS] [PROMPT]\n"
}

fn run_features(command: FeaturesCommand) -> BrowserRunResult {
    match command.command {
        FeaturesSubcommand::List => BrowserRunResult::stdout(
            [
                "browser_wasm_cli\tstable\ttrue",
                "browser_exec_bridge\tworker bridge available\ttrue",
                "browser_tui\twasm ratatui frame renderer\ttrue",
                "browser_app_server\tunder development\tfalse",
            ]
            .join("\n")
                + "\n",
            0,
        ),
        FeaturesSubcommand::Enable(feature) => BrowserRunResult::stdout(
            format!(
                "Feature `{}` recorded for this browser command invocation.\n",
                feature.feature
            ),
            0,
        ),
        FeaturesSubcommand::Disable(feature) => BrowserRunResult::stdout(
            format!(
                "Feature `{}` disabled for this browser command invocation.\n",
                feature.feature
            ),
            0,
        ),
    }
}

fn run_doctor(options: BrowserRunOptions) -> BrowserRunResult {
    let mut stdout = String::from("Codex browser WASM doctor\n");
    stdout.push_str("wasm module: loaded\n");
    stdout.push_str("argv parser: available\n");
    stdout.push_str("browser exec bridge: available in the Web Worker host bridge\n");
    stdout.push_str("browser TUI renderer: ratatui wasm frame renderer available\n");
    stdout.push_str("native process/socket app-server bridge: unavailable\n");
    if let Some(cwd) = options.cwd {
        stdout.push_str(&format!("cwd: {cwd}\n"));
    }
    BrowserRunResult::stdout(stdout, 0)
}

fn browser_tui_options(options: &BrowserRunOptions) -> BrowserTuiSessionOptions {
    BrowserTuiSessionOptions {
        cwd: options.cwd.clone(),
        env: options.env.clone(),
        terminal_width: options.terminal_width,
        terminal_height: options.terminal_height,
    }
}

fn browser_tui_result(result: Result<BrowserTuiRunResult, String>) -> BrowserRunResult {
    match result {
        Ok(result) => BrowserRunResult::browser_tui(result),
        Err(message) => BrowserRunResult::stderr(message, 1),
    }
}

fn parse_browser_tui_input_arg(args: Vec<String>) -> Result<Option<String>, BrowserRunResult> {
    if args.is_empty() {
        return Ok(None);
    }
    parse_browser_tui_required_input_arg(args).map(Some)
}

fn parse_browser_tui_required_input_arg(args: Vec<String>) -> Result<String, BrowserRunResult> {
    let mut input: Option<String> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--input"));
                };
                input = Some(value.clone());
                index += 2;
            }
            other => {
                return Err(BrowserRunResult::stderr(
                    format!("error: unsupported browser TUI debug option '{other}'\n"),
                    2,
                ));
            }
        }
    }
    Ok(input.unwrap_or_default())
}

fn parse_browser_tui_event_args(args: Vec<String>) -> Result<BrowserTuiEvent, BrowserRunResult> {
    let mut event_type = "key".to_string();
    let mut name = String::new();
    let mut text = String::new();
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--type" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--type"));
                };
                event_type = value.clone();
                index += 2;
            }
            "--name" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--name"));
                };
                name = value.clone();
                index += 2;
            }
            "--text" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--text"));
                };
                text = value.clone();
                index += 2;
            }
            "--ctrl" => {
                ctrl = true;
                index += 1;
            }
            "--alt" | "--meta" => {
                alt = true;
                index += 1;
            }
            "--shift" => {
                shift = true;
                index += 1;
            }
            other => {
                return Err(BrowserRunResult::stderr(
                    format!("error: unsupported browser TUI event option '{other}'\n"),
                    2,
                ));
            }
        }
    }

    match event_type.as_str() {
        "key" => Ok(BrowserTuiEvent::Key(BrowserTuiKeyEvent {
            code: parse_browser_tui_key_code(&name, &text),
            ctrl,
            alt,
            shift,
        })),
        "paste" => Ok(BrowserTuiEvent::Paste(text)),
        "resize" => Ok(BrowserTuiEvent::Resize),
        "draw" => Ok(BrowserTuiEvent::Draw),
        other => Err(BrowserRunResult::stderr(
            format!("error: unsupported browser TUI event type '{other}'\n"),
            2,
        )),
    }
}

fn parse_browser_tui_key_code(name: &str, text: &str) -> BrowserTuiKeyCode {
    let normalized = name.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" | "char" => BrowserTuiKeyCode::Char(text.to_string()),
        "return" | "enter" => BrowserTuiKeyCode::Enter,
        "backspace" => BrowserTuiKeyCode::Backspace,
        "delete" | "del" => BrowserTuiKeyCode::Delete,
        "escape" | "esc" => BrowserTuiKeyCode::Escape,
        "tab" => BrowserTuiKeyCode::Tab,
        "left" => BrowserTuiKeyCode::Left,
        "right" => BrowserTuiKeyCode::Right,
        "up" => BrowserTuiKeyCode::Up,
        "down" => BrowserTuiKeyCode::Down,
        "home" => BrowserTuiKeyCode::Home,
        "end" => BrowserTuiKeyCode::End,
        "c" | "d" if !text.is_empty() => BrowserTuiKeyCode::Char(text.to_string()),
        "c" | "d" => BrowserTuiKeyCode::Char(normalized),
        other => {
            if !text.is_empty() {
                BrowserTuiKeyCode::Char(text.to_string())
            } else {
                BrowserTuiKeyCode::Unknown(other.to_string())
            }
        }
    }
}

fn parse_browser_tui_result_args(
    args: Vec<String>,
) -> Result<BrowserTuiAppendResult, BrowserRunResult> {
    let mut kind = BrowserTuiResultKind::Exec;
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut exit_code = 0;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--kind" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--kind"));
                };
                kind = match value.as_str() {
                    "exec" => BrowserTuiResultKind::Exec,
                    "shell" => BrowserTuiResultKind::Shell,
                    other => {
                        return Err(BrowserRunResult::stderr(
                            format!("error: unsupported browser TUI result kind '{other}'\n"),
                            2,
                        ));
                    }
                };
                index += 2;
            }
            "--stdout" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--stdout"));
                };
                stdout = value.clone();
                index += 2;
            }
            "--stderr" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--stderr"));
                };
                stderr = value.clone();
                index += 2;
            }
            "--exit-code" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--exit-code"));
                };
                exit_code = match value.parse::<i32>() {
                    Ok(value) => value,
                    Err(_) => {
                        return Err(BrowserRunResult::stderr(
                            "error: --exit-code must be an integer\n",
                            2,
                        ));
                    }
                };
                index += 2;
            }
            other => {
                return Err(BrowserRunResult::stderr(
                    format!("error: unsupported browser TUI result option '{other}'\n"),
                    2,
                ));
            }
        }
    }

    Ok(BrowserTuiAppendResult {
        kind,
        stdout,
        stderr,
        exit_code,
    })
}

fn parse_browser_tui_agent_message_args(args: Vec<String>) -> Result<String, BrowserRunResult> {
    let mut message: Option<String> = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--text" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--text"));
                };
                message = Some(value.clone());
                index += 2;
            }
            other => {
                return Err(BrowserRunResult::stderr(
                    format!("error: unsupported browser TUI agent message option '{other}'\n"),
                    2,
                ));
            }
        }
    }

    let Some(message) = message else {
        return Err(missing_debug_flag_value("--text"));
    };

    Ok(message)
}

fn parse_browser_tui_plan_update_args(
    args: Vec<String>,
) -> Result<UpdatePlanArgs, BrowserRunResult> {
    let mut explanation: Option<String> = None;
    let mut plan = Vec::new();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--explanation" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--explanation"));
                };
                explanation = Some(value.clone());
                index += 2;
            }
            "--step" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--step"));
                };
                plan.push(parse_browser_tui_plan_step(value)?);
                index += 2;
            }
            other => {
                return Err(BrowserRunResult::stderr(
                    format!("error: unsupported browser TUI plan update option '{other}'\n"),
                    2,
                ));
            }
        }
    }

    Ok(UpdatePlanArgs { explanation, plan })
}

fn parse_browser_tui_notification_args(args: Vec<String>) -> Result<String, BrowserRunResult> {
    let mut notification_json: Option<String> = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(missing_debug_flag_value("--json"));
                };
                notification_json = Some(value.clone());
                index += 2;
            }
            other => {
                return Err(BrowserRunResult::stderr(
                    format!("error: unsupported browser TUI notification option '{other}'\n"),
                    2,
                ));
            }
        }
    }

    let Some(notification_json) = notification_json else {
        return Err(missing_debug_flag_value("--json"));
    };

    Ok(notification_json)
}

fn parse_browser_tui_plan_step(value: &str) -> Result<PlanItemArg, BrowserRunResult> {
    let Some((status, step)) = value.split_once(':') else {
        return Err(BrowserRunResult::stderr(
            "error: --step must be formatted as <status>:<text>\n",
            2,
        ));
    };
    let step = step.trim();
    if step.is_empty() {
        return Err(BrowserRunResult::stderr(
            "error: --step text must not be empty\n",
            2,
        ));
    }

    Ok(PlanItemArg {
        step: step.to_string(),
        status: parse_browser_tui_plan_status(status)?,
    })
}

fn parse_browser_tui_plan_status(value: &str) -> Result<StepStatus, BrowserRunResult> {
    match value.trim() {
        "pending" => Ok(StepStatus::Pending),
        "in_progress" | "in-progress" | "inProgress" | "active" => Ok(StepStatus::InProgress),
        "completed" | "complete" | "done" => Ok(StepStatus::Completed),
        other => Err(BrowserRunResult::stderr(
            format!("error: unsupported browser TUI plan step status '{other}'\n"),
            2,
        )),
    }
}

fn missing_debug_flag_value(flag: &str) -> BrowserRunResult {
    BrowserRunResult::stderr(format!("error: a value is required for '{flag}'\n"), 2)
}

fn unsupported(command: &str, args: Vec<String>) -> BrowserRunResult {
    let args = if args.is_empty() {
        String::new()
    } else {
        format!(" {}", args.join(" "))
    };
    BrowserRunResult::stderr(
        format!(
            "codex {command}{args} is recognized by the browser WASM CLI, but this subcommand still depends on native Codex runtime services.\n\
             Required browser work: route filesystem, process execution, auth, networking, and app-server/TUI transport through almostnode host bridges.\n"
        ),
        78,
    )
}

fn render_completion_placeholder(shell: String) -> String {
    format!(
        "# codex completion for {shell}\n\
         # The browser WASM CLI can parse completion requests, but full clap completion generation is not bundled yet.\n"
    )
}

fn session_args(session_id: Option<String>, last: bool, all: bool) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(session_id) = session_id {
        args.push(session_id);
    }
    if last {
        args.push("--last".to_string());
    }
    if all {
        args.push("--all".to_string());
    }
    args
}

impl BrowserRunResult {
    pub(crate) fn stdout(stdout: impl Into<String>, exit_code: i32) -> BrowserRunResult {
        BrowserRunResult {
            stdout: stdout.into(),
            stderr: String::new(),
            exit_code,
            env: Vec::new(),
            browser_exec: None,
            browser_login: None,
            browser_tui: None,
        }
    }

    pub(crate) fn stderr(stderr: impl Into<String>, exit_code: i32) -> BrowserRunResult {
        BrowserRunResult {
            stdout: String::new(),
            stderr: stderr.into(),
            exit_code,
            env: Vec::new(),
            browser_exec: None,
            browser_login: None,
            browser_tui: None,
        }
    }

    pub(crate) fn with_env(mut self, env: Vec<(String, String)>) -> BrowserRunResult {
        self.env = env;
        self
    }

    fn browser_exec(plan: BrowserExecPlan) -> BrowserRunResult {
        BrowserRunResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            env: Vec::new(),
            browser_exec: Some(plan),
            browser_login: None,
            browser_tui: None,
        }
    }

    fn browser_login(request: BrowserLoginRequest) -> BrowserRunResult {
        BrowserRunResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            env: Vec::new(),
            browser_exec: None,
            browser_login: Some(request),
            browser_tui: None,
        }
    }

    fn browser_tui(result: BrowserTuiRunResult) -> BrowserRunResult {
        BrowserRunResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            env: Vec::new(),
            browser_exec: None,
            browser_login: None,
            browser_tui: Some(result),
        }
    }
}

fn env_value<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a str> {
    env.iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.as_str())
}

fn browser_cli_version(options: &BrowserRunOptions) -> &str {
    env_value(&options.env, "CODEX_CLI_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_uses_codex_command_surface() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(vec!["--help".to_string()], BrowserRunOptions::default());
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Codex CLI"));
        assert!(result.stdout.contains("exec"));
    }

    #[test]
    fn version_uses_browser_release_env_override() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["--version".to_string()],
            BrowserRunOptions {
                env: vec![("CODEX_CLI_VERSION".to_string(), "0.137.0".to_string())],
                ..BrowserRunOptions::default()
            },
        );
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "codex 0.137.0\n");
    }

    #[test]
    fn login_status_reads_browser_env() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["login".to_string(), "status".to_string()],
            BrowserRunOptions {
                env: vec![("OPENAI_API_KEY".to_string(), "test-key".to_string())],
                ..BrowserRunOptions::default()
            },
        );
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("OPENAI_API_KEY"));

        let result = session.run(
            vec!["login".to_string(), "status".to_string()],
            BrowserRunOptions {
                env: vec![("CODEX_API_KEY".to_string(), "test-key".to_string())],
                ..BrowserRunOptions::default()
            },
        );
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("CODEX_API_KEY"));
    }

    #[test]
    fn login_with_api_key_returns_browser_session_env() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["login".to_string(), "--with-api-key".to_string()],
            BrowserRunOptions {
                stdin: Some("sk-test\n".to_string()),
                ..BrowserRunOptions::default()
            },
        );
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Stored OpenAI API key"));
        assert_eq!(
            result.env,
            vec![
                ("OPENAI_API_KEY".to_string(), "sk-test".to_string()),
                ("CODEX_API_KEY".to_string(), "sk-test".to_string()),
            ]
        );

        let status = session.run(
            vec!["login".to_string(), "status".to_string()],
            BrowserRunOptions::default(),
        );
        assert_eq!(status.exit_code, 0);
        assert!(status.stdout.contains("stdin api key/browser session"));
    }

    #[test]
    fn exec_returns_browser_host_plan() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["exec".to_string(), "hello".to_string()],
            BrowserRunOptions::default(),
        );
        assert_eq!(result.exit_code, 0);
        let plan = result.browser_exec.expect("exec should produce host plan");
        assert_eq!(plan.prompt, "hello");
        assert_eq!(plan.model, DEFAULT_BROWSER_CODEX_MODEL);
    }

    #[test]
    fn interactive_start_returns_browser_tui_frame() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            Vec::new(),
            BrowserRunOptions {
                cwd: Some("/workspace".to_string()),
                terminal_width: Some(80),
                terminal_height: Some(24),
                ..BrowserRunOptions::default()
            },
        );
        assert_eq!(result.exit_code, 0);
        let tui = result.browser_tui.expect("interactive should render TUI");
        assert!(tui.ansi.contains("OpenAI Codex"));
        assert_eq!(tui.action, BrowserTuiAction::None);
    }

    #[test]
    fn interactive_submit_classifies_shell_command_in_rust() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["debug".to_string(), "browser-tui-start".to_string()],
            BrowserRunOptions::default(),
        );
        assert_eq!(result.exit_code, 0);

        let result = session.run(
            vec![
                "debug".to_string(),
                "browser-tui-input".to_string(),
                "--input".to_string(),
                "!ls".to_string(),
            ],
            BrowserRunOptions::default(),
        );
        assert_eq!(result.exit_code, 0);

        let result = session.run(
            vec!["debug".to_string(), "browser-tui-submit".to_string()],
            BrowserRunOptions::default(),
        );
        assert_eq!(result.exit_code, 0);
        let tui = result.browser_tui.expect("submit should render TUI");
        assert_eq!(
            tui.action,
            BrowserTuiAction::Shell {
                command: "ls".to_string(),
            }
        );
    }

    #[test]
    fn interactive_submit_uses_upstream_slash_command_parser() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["debug".to_string(), "browser-tui-start".to_string()],
            BrowserRunOptions {
                cwd: Some("/workspace".to_string()),
                ..BrowserRunOptions::default()
            },
        );
        assert_eq!(result.exit_code, 0);

        let result = session.run(
            vec![
                "debug".to_string(),
                "browser-tui-input".to_string(),
                "--input".to_string(),
                "/status".to_string(),
            ],
            BrowserRunOptions {
                cwd: Some("/workspace".to_string()),
                ..BrowserRunOptions::default()
            },
        );
        assert_eq!(result.exit_code, 0);

        let result = session.run(
            vec!["debug".to_string(), "browser-tui-submit".to_string()],
            BrowserRunOptions {
                cwd: Some("/workspace".to_string()),
                terminal_width: Some(100),
                terminal_height: Some(28),
                ..BrowserRunOptions::default()
            },
        );
        assert_eq!(result.exit_code, 0);
        let tui = result.browser_tui.expect("status should render TUI");
        assert_eq!(tui.action, BrowserTuiAction::None);
        assert!(tui.ansi.contains("Codex browser session status"));
        assert!(tui.ansi.contains("/workspace"));
    }

    #[test]
    fn interactive_init_slash_command_submits_upstream_init_prompt() {
        let mut session = BrowserCodexCliSession::new();
        let auth_options = || BrowserRunOptions {
            env: vec![("OPENAI_API_KEY".to_string(), "test-key".to_string())],
            ..BrowserRunOptions::default()
        };
        let result = session.run(
            vec!["debug".to_string(), "browser-tui-start".to_string()],
            auth_options(),
        );
        assert_eq!(result.exit_code, 0);

        let result = session.run(
            vec![
                "debug".to_string(),
                "browser-tui-input".to_string(),
                "--input".to_string(),
                "/init".to_string(),
            ],
            auth_options(),
        );
        assert_eq!(result.exit_code, 0);

        let result = session.run(
            vec!["debug".to_string(), "browser-tui-submit".to_string()],
            auth_options(),
        );
        assert_eq!(result.exit_code, 0);
        let tui = result.browser_tui.expect("init should render TUI");
        assert_eq!(
            tui.action,
            BrowserTuiAction::Exec {
                prompt: include_str!("../../tui/prompt_for_init_command.md").to_string(),
            }
        );
    }

    #[test]
    fn unsupported_slash_commands_are_not_sent_as_user_prompts() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["debug".to_string(), "browser-tui-start".to_string()],
            BrowserRunOptions::default(),
        );
        assert_eq!(result.exit_code, 0);

        let result = session.run(
            vec![
                "debug".to_string(),
                "browser-tui-input".to_string(),
                "--input".to_string(),
                "/model".to_string(),
            ],
            BrowserRunOptions::default(),
        );
        assert_eq!(result.exit_code, 0);

        let result = session.run(
            vec!["debug".to_string(), "browser-tui-submit".to_string()],
            BrowserRunOptions {
                terminal_width: Some(100),
                terminal_height: Some(28),
                ..BrowserRunOptions::default()
            },
        );
        assert_eq!(result.exit_code, 0);
        let tui = result.browser_tui.expect("model should render TUI");
        assert_eq!(tui.action, BrowserTuiAction::None);
        assert!(tui.ansi.contains("native /model picker is recognized"));
    }

    #[test]
    fn interactive_key_events_are_routed_through_tui_crate() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["debug".to_string(), "browser-tui-start".to_string()],
            BrowserRunOptions::default(),
        );
        assert_eq!(result.exit_code, 0);

        for text in ["!", "l", "s"] {
            let result = session.run(
                vec![
                    "debug".to_string(),
                    "browser-tui-event".to_string(),
                    "--type".to_string(),
                    "key".to_string(),
                    "--name".to_string(),
                    "char".to_string(),
                    "--text".to_string(),
                    text.to_string(),
                ],
                BrowserRunOptions::default(),
            );
            assert_eq!(result.exit_code, 0);
        }

        let result = session.run(
            vec![
                "debug".to_string(),
                "browser-tui-event".to_string(),
                "--type".to_string(),
                "key".to_string(),
                "--name".to_string(),
                "return".to_string(),
            ],
            BrowserRunOptions::default(),
        );

        assert_eq!(result.exit_code, 0);
        let tui = result.browser_tui.expect("enter should render TUI");
        assert_eq!(
            tui.action,
            BrowserTuiAction::Shell {
                command: "ls".to_string(),
            }
        );
    }

    #[test]
    fn debug_plan_update_renders_real_tui_history_cell() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["debug".to_string(), "browser-tui-start".to_string()],
            BrowserRunOptions::default(),
        );
        assert_eq!(result.exit_code, 0);

        let result = session.run(
            vec![
                "debug".to_string(),
                "browser-tui-plan-update".to_string(),
                "--explanation".to_string(),
                "Browser wasm plan update".to_string(),
                "--step".to_string(),
                "completed:Compile the forked Codex TUI renderer for wasm".to_string(),
                "--step".to_string(),
                "inProgress:Route browser terminal input through the wasm frame loop".to_string(),
            ],
            BrowserRunOptions {
                terminal_width: Some(100),
                terminal_height: Some(28),
                ..BrowserRunOptions::default()
            },
        );

        assert_eq!(result.exit_code, 0);
        let tui = result.browser_tui.expect("plan update should render TUI");
        assert_eq!(tui.action, BrowserTuiAction::None);
        assert!(tui.ansi.contains("Updated Plan"));
        assert!(
            tui.ansi
                .contains("Compile the forked Codex TUI renderer for wasm")
        );
        assert!(
            tui.ansi
                .contains("Route browser terminal input through the wasm frame loop")
        );
    }

    #[test]
    fn debug_agent_message_renders_real_tui_history_cell() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["debug".to_string(), "browser-tui-start".to_string()],
            BrowserRunOptions::default(),
        );
        assert_eq!(result.exit_code, 0);

        let result = session.run(
            vec![
                "debug".to_string(),
                "browser-tui-agent-message".to_string(),
                "--text".to_string(),
                "**Native assistant message**\n\n- rendered by Codex markdown".to_string(),
            ],
            BrowserRunOptions {
                terminal_width: Some(100),
                terminal_height: Some(28),
                ..BrowserRunOptions::default()
            },
        );

        assert_eq!(result.exit_code, 0);
        let tui = result.browser_tui.expect("agent message should render TUI");
        assert_eq!(tui.action, BrowserTuiAction::None);
        assert!(tui.ansi.contains("Native assistant message"));
        assert!(tui.ansi.contains("rendered by Codex markdown"));
    }

    #[test]
    fn native_only_subcommands_are_explicit() {
        let mut session = BrowserCodexCliSession::new();
        let result = session.run(
            vec!["review".to_string(), "hello".to_string()],
            BrowserRunOptions::default(),
        );
        assert_eq!(result.exit_code, 78);
        assert!(result.stderr.contains("recognized by the browser WASM CLI"));
    }
}
