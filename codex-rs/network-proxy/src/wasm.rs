use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use clap::Parser;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone, Parser)]
#[command(name = "codex-network-proxy", about = "Codex network sandbox proxy")]
pub struct Args {}

pub const CUSTOM_CA_ENV_KEYS: [&str; 10] = [
    "CODEX_CA_CERTIFICATE",
    "SSL_CERT_FILE",
    "REQUESTS_CA_BUNDLE",
    "CURL_CA_BUNDLE",
    "NODE_EXTRA_CA_CERTS",
    "GIT_SSL_CAINFO",
    "PIP_CERT",
    "BUNDLE_SSL_CA_CERT",
    "npm_config_cafile",
    "NPM_CONFIG_CAFILE",
];

pub const PROXY_URL_ENV_KEYS: &[&str] = &[
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "http_proxy",
    "https_proxy",
    "ALL_PROXY",
    "all_proxy",
];
pub const ALL_PROXY_ENV_KEYS: &[&str] = &["ALL_PROXY", "all_proxy"];
pub const PROXY_ACTIVE_ENV_KEY: &str = "CODEX_NETWORK_PROXY_ACTIVE";
pub const ALLOW_LOCAL_BINDING_ENV_KEY: &str = "CODEX_NETWORK_ALLOW_LOCAL_BINDING";
pub const PROXY_ENV_KEYS: &[&str] = &[
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "http_proxy",
    "https_proxy",
    "ALL_PROXY",
    "all_proxy",
    "NO_PROXY",
    "no_proxy",
];
pub const NO_PROXY_ENV_KEYS: &[&str] = &["NO_PROXY", "no_proxy"];
pub const DEFAULT_NO_PROXY_VALUE: &str = "localhost,127.0.0.1,::1";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct NetworkProxyConfig {
    #[serde(default)]
    pub network: NetworkProxySettings,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum NetworkDomainPermission {
    None,
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkDomainPermissionEntry {
    pub pattern: String,
    pub permission: NetworkDomainPermission,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkDomainPermissions {
    pub entries: Vec<NetworkDomainPermissionEntry>,
}

impl Serialize for NetworkDomainPermissions {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.effective_entries()
            .into_iter()
            .map(|entry| (entry.pattern, entry.permission))
            .collect::<BTreeMap<_, _>>()
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NetworkDomainPermissions {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries = BTreeMap::<String, NetworkDomainPermission>::deserialize(deserializer)?
            .into_iter()
            .map(|(pattern, permission)| NetworkDomainPermissionEntry {
                pattern,
                permission,
            })
            .collect();
        Ok(Self { entries })
    }
}

impl NetworkDomainPermissions {
    fn effective_entries(&self) -> Vec<NetworkDomainPermissionEntry> {
        let mut order = Vec::new();
        let mut effective_permissions = BTreeMap::new();

        for entry in &self.entries {
            if !effective_permissions.contains_key(&entry.pattern) {
                order.push(entry.pattern.clone());
            }
            let permission = effective_permissions
                .entry(entry.pattern.clone())
                .or_insert(entry.permission);
            if entry.permission > *permission {
                *permission = entry.permission;
            }
        }

        order
            .into_iter()
            .filter_map(|pattern| {
                effective_permissions.remove(&pattern).map(|permission| {
                    NetworkDomainPermissionEntry {
                        pattern,
                        permission,
                    }
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NetworkUnixSocketPermission {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct NetworkUnixSocketPermissions {
    #[serde(flatten)]
    pub entries: BTreeMap<String, NetworkUnixSocketPermission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct NetworkProxySettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_proxy_url")]
    pub proxy_url: String,
    pub enable_socks5: bool,
    #[serde(default = "default_socks_url")]
    pub socks_url: String,
    pub enable_socks5_udp: bool,
    pub allow_upstream_proxy: bool,
    #[serde(default)]
    pub dangerously_allow_non_loopback_proxy: bool,
    #[serde(default)]
    pub dangerously_allow_all_unix_sockets: bool,
    #[serde(default)]
    pub mode: NetworkMode,
    #[serde(default)]
    pub domains: Option<NetworkDomainPermissions>,
    #[serde(default)]
    pub unix_sockets: Option<NetworkUnixSocketPermissions>,
    pub allow_local_binding: bool,
    #[serde(default)]
    pub mitm: bool,
    #[serde(default)]
    pub mitm_hooks: Vec<MitmHookConfig>,
}

impl Default for NetworkProxySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            proxy_url: default_proxy_url(),
            enable_socks5: true,
            socks_url: default_socks_url(),
            enable_socks5_udp: true,
            allow_upstream_proxy: true,
            dangerously_allow_non_loopback_proxy: false,
            dangerously_allow_all_unix_sockets: false,
            mode: NetworkMode::default(),
            domains: None,
            unix_sockets: None,
            allow_local_binding: false,
            mitm: false,
            mitm_hooks: Vec::new(),
        }
    }
}

impl NetworkProxySettings {
    pub fn allowed_domains(&self) -> Option<Vec<String>> {
        self.domain_entries(NetworkDomainPermission::Allow)
    }

    pub fn denied_domains(&self) -> Option<Vec<String>> {
        self.domain_entries(NetworkDomainPermission::Deny)
    }

    fn domain_entries(&self, permission: NetworkDomainPermission) -> Option<Vec<String>> {
        self.domains
            .as_ref()
            .map(|domains| {
                domains
                    .effective_entries()
                    .iter()
                    .filter(|entry| entry.permission == permission)
                    .map(|entry| entry.pattern.clone())
                    .collect()
            })
            .filter(|entries: &Vec<String>| !entries.is_empty())
    }

    pub fn allow_unix_sockets(&self) -> Vec<String> {
        self.unix_sockets
            .as_ref()
            .map(|unix_sockets| {
                unix_sockets
                    .entries
                    .iter()
                    .filter(|(_, permission)| {
                        matches!(permission, NetworkUnixSocketPermission::Allow)
                    })
                    .map(|(path, _)| path.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn set_allowed_domains(&mut self, allowed_domains: Vec<String>) {
        self.set_domain_entries(allowed_domains, NetworkDomainPermission::Allow);
    }

    pub fn set_denied_domains(&mut self, denied_domains: Vec<String>) {
        self.set_domain_entries(denied_domains, NetworkDomainPermission::Deny);
    }

    pub fn upsert_domain_permission(
        &mut self,
        host: String,
        permission: NetworkDomainPermission,
        normalize: impl Fn(&str) -> String,
    ) {
        let mut domains = self.domains.take().unwrap_or_default();
        let normalized_host = normalize(&host);
        domains
            .entries
            .retain(|entry| normalize(&entry.pattern) != normalized_host);
        domains.entries.push(NetworkDomainPermissionEntry {
            pattern: host,
            permission,
        });
        self.domains = (!domains.entries.is_empty()).then_some(domains);
    }

    pub fn set_allow_unix_sockets(&mut self, allow_unix_sockets: Vec<String>) {
        let mut unix_sockets = self.unix_sockets.take().unwrap_or_default();
        unix_sockets
            .entries
            .retain(|_, existing| *existing != NetworkUnixSocketPermission::Allow);
        for entry in allow_unix_sockets {
            unix_sockets
                .entries
                .insert(entry, NetworkUnixSocketPermission::Allow);
        }
        self.unix_sockets = (!unix_sockets.entries.is_empty()).then_some(unix_sockets);
    }

    fn set_domain_entries(&mut self, entries: Vec<String>, permission: NetworkDomainPermission) {
        let mut domains = self.domains.take().unwrap_or_default();
        domains
            .entries
            .retain(|entry| entry.permission != permission);
        for entry in entries {
            domains.entries.push(NetworkDomainPermissionEntry {
                pattern: entry,
                permission,
            });
        }
        self.domains = (!domains.entries.is_empty()).then_some(domains);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
    Limited,
    #[default]
    Full,
}

impl NetworkMode {
    pub fn allows_method(self, method: &str) -> bool {
        match self {
            Self::Full => true,
            Self::Limited => matches!(method, "GET" | "HEAD" | "OPTIONS"),
        }
    }
}

fn default_proxy_url() -> String {
    "http://127.0.0.1:3128".to_string()
}

fn default_socks_url() -> String {
    "http://127.0.0.1:8081".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct MitmHookConfig {
    pub host: String,
    #[serde(rename = "match", default)]
    pub matcher: MitmHookMatchConfig,
    #[serde(default)]
    pub actions: MitmHookActionsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct MitmHookMatchConfig {
    pub methods: Vec<String>,
    pub path_prefixes: Vec<String>,
    pub query: BTreeMap<String, Vec<String>>,
    pub headers: BTreeMap<String, Vec<String>>,
    pub body: Option<MitmHookBodyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct MitmHookActionsConfig {
    pub strip_request_headers: Vec<String>,
    pub inject_request_headers: Vec<InjectedHeaderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct InjectedHeaderConfig {
    pub name: String,
    pub secret_env_var: Option<String>,
    pub secret_file: Option<String>,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct MitmHookBodyConfig(pub serde_json::Value);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NetworkProxyAuditMetadata {
    pub conversation_id: Option<String>,
    pub app_version: Option<String>,
    pub user_account_id: Option<String>,
    pub auth_mode: Option<String>,
    pub originator: Option<String>,
    pub user_email: Option<String>,
    pub terminal_type: Option<String>,
    pub model: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NetworkProxyConstraints {
    pub enabled: Option<bool>,
    pub mode: Option<NetworkMode>,
    pub allow_upstream_proxy: Option<bool>,
    pub dangerously_allow_non_loopback_proxy: Option<bool>,
    pub dangerously_allow_all_unix_sockets: Option<bool>,
    pub allowed_domains: Option<Vec<String>>,
    pub allowlist_expansion_enabled: Option<bool>,
    pub denied_domains: Option<Vec<String>>,
    pub denylist_expansion_enabled: Option<bool>,
    pub allow_unix_sockets: Option<Vec<String>>,
    pub allow_local_binding: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartialNetworkProxyConfig {
    #[serde(default)]
    pub network: PartialNetworkConfig,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PartialNetworkConfig {
    pub enabled: Option<bool>,
    pub mode: Option<NetworkMode>,
    pub allow_upstream_proxy: Option<bool>,
    pub dangerously_allow_non_loopback_proxy: Option<bool>,
    pub dangerously_allow_all_unix_sockets: Option<bool>,
    #[serde(default)]
    pub domains: Option<NetworkDomainPermissions>,
    #[serde(default)]
    pub unix_sockets: Option<NetworkUnixSocketPermissions>,
    pub allow_local_binding: Option<bool>,
    pub mitm: Option<bool>,
    #[serde(default)]
    pub mitm_hooks: Option<Vec<MitmHookConfig>>,
}

#[derive(Clone)]
pub struct ConfigState {
    pub config: NetworkProxyConfig,
    pub constraints: NetworkProxyConstraints,
    pub blocked: VecDeque<BlockedRequest>,
    pub blocked_total: u64,
}

pub fn build_config_state(
    config: NetworkProxyConfig,
    constraints: NetworkProxyConstraints,
) -> Result<ConfigState> {
    Ok(ConfigState {
        config,
        constraints,
        blocked: VecDeque::new(),
        blocked_total: 0,
    })
}

pub fn validate_policy_against_constraints(
    _config: &NetworkProxyConfig,
    _constraints: &NetworkProxyConstraints,
) -> std::result::Result<(), NetworkProxyConstraintError> {
    Ok(())
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NetworkProxyConstraintError {
    #[error("invalid value for {field_name}: {candidate}; allowed: {allowed}")]
    InvalidValue {
        field_name: &'static str,
        candidate: String,
        allowed: String,
    },
}

impl NetworkProxyConstraintError {
    pub fn into_anyhow(self) -> anyhow::Error {
        anyhow!(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostBlockReason {
    Denied,
    NotAllowed,
    NotAllowedLocal,
}

impl HostBlockReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Denied => "denied",
            Self::NotAllowed => "not_allowed",
            Self::NotAllowedLocal => "not_allowed_local",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostBlockDecision {
    Allowed,
    Blocked(HostBlockReason),
}

#[derive(Clone, Debug, Serialize)]
pub struct BlockedRequest {
    pub host: String,
    pub reason: String,
    pub client: Option<String>,
    pub method: Option<String>,
    pub mode: Option<NetworkMode>,
    pub protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    pub timestamp: i64,
}

pub struct BlockedRequestArgs {
    pub host: String,
    pub reason: String,
    pub client: Option<String>,
    pub method: Option<String>,
    pub mode: Option<NetworkMode>,
    pub protocol: String,
    pub decision: Option<String>,
    pub source: Option<String>,
    pub port: Option<u16>,
}

impl BlockedRequest {
    pub fn new(args: BlockedRequestArgs) -> Self {
        Self {
            host: args.host,
            reason: args.reason,
            client: args.client,
            method: args.method,
            mode: args.mode,
            protocol: args.protocol,
            decision: args.decision,
            source: args.source,
            port: args.port,
            timestamp: 0,
        }
    }
}

#[async_trait(?Send)]
pub trait ConfigReloader: 'static {
    fn source_label(&self) -> String;
    async fn maybe_reload(&self) -> Result<Option<ConfigState>>;
    async fn reload_now(&self) -> Result<ConfigState>;
}

#[async_trait(?Send)]
pub trait BlockedRequestObserver: 'static {
    async fn on_blocked_request(&self, request: BlockedRequest);
}

#[async_trait(?Send)]
impl<O: BlockedRequestObserver + ?Sized> BlockedRequestObserver for Arc<O> {
    async fn on_blocked_request(&self, request: BlockedRequest) {
        (**self).on_blocked_request(request).await
    }
}

#[async_trait(?Send)]
impl<F, Fut> BlockedRequestObserver for F
where
    F: Fn(BlockedRequest) -> Fut + 'static,
    Fut: Future<Output = ()>,
{
    async fn on_blocked_request(&self, request: BlockedRequest) {
        (self)(request).await
    }
}

#[derive(Clone)]
pub struct NetworkProxyState {
    state: Arc<RwLock<ConfigState>>,
    audit_metadata: NetworkProxyAuditMetadata,
}

impl std::fmt::Debug for NetworkProxyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkProxyState").finish_non_exhaustive()
    }
}

impl NetworkProxyState {
    pub fn with_reloader(state: ConfigState, _reloader: Arc<dyn ConfigReloader>) -> Self {
        Self::with_reloader_and_audit_metadata(
            state,
            _reloader,
            NetworkProxyAuditMetadata::default(),
        )
    }

    pub fn with_reloader_and_blocked_observer(
        state: ConfigState,
        reloader: Arc<dyn ConfigReloader>,
        _blocked_request_observer: Option<Arc<dyn BlockedRequestObserver>>,
    ) -> Self {
        Self::with_reloader(state, reloader)
    }

    pub fn with_reloader_and_audit_metadata(
        state: ConfigState,
        _reloader: Arc<dyn ConfigReloader>,
        audit_metadata: NetworkProxyAuditMetadata,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
            audit_metadata,
        }
    }

    pub fn with_reloader_and_audit_metadata_and_blocked_observer(
        state: ConfigState,
        reloader: Arc<dyn ConfigReloader>,
        audit_metadata: NetworkProxyAuditMetadata,
        _blocked_request_observer: Option<Arc<dyn BlockedRequestObserver>>,
    ) -> Self {
        Self::with_reloader_and_audit_metadata(state, reloader, audit_metadata)
    }

    pub async fn set_blocked_request_observer(
        &self,
        _blocked_request_observer: Option<Arc<dyn BlockedRequestObserver>>,
    ) {
    }

    pub fn audit_metadata(&self) -> &NetworkProxyAuditMetadata {
        &self.audit_metadata
    }

    pub async fn current_cfg(&self) -> Result<NetworkProxyConfig> {
        Ok(self.state.read().unwrap().config.clone())
    }

    pub async fn current_patterns(&self) -> Result<(Vec<String>, Vec<String>)> {
        let cfg = self.current_cfg().await?;
        Ok((
            cfg.network.allowed_domains().unwrap_or_default(),
            cfg.network.denied_domains().unwrap_or_default(),
        ))
    }

    pub async fn enabled(&self) -> Result<bool> {
        Ok(self.current_cfg().await?.network.enabled)
    }

    pub async fn force_reload(&self) -> Result<()> {
        Ok(())
    }

    pub async fn replace_config_state(&self, new_state: ConfigState) -> Result<()> {
        *self.state.write().unwrap() = new_state;
        Ok(())
    }

    pub async fn host_blocked(&self, _host: &str, _port: u16) -> Result<HostBlockDecision> {
        Ok(HostBlockDecision::Allowed)
    }

    pub async fn record_blocked(&self, entry: BlockedRequest) -> Result<()> {
        let mut guard = self.state.write().unwrap();
        guard.blocked.push_back(entry);
        guard.blocked_total += 1;
        Ok(())
    }

    pub async fn blocked_snapshot(&self) -> Result<Vec<BlockedRequest>> {
        Ok(self.state.read().unwrap().blocked.iter().cloned().collect())
    }

    pub async fn drain_blocked(&self) -> Result<Vec<BlockedRequest>> {
        Ok(self.state.write().unwrap().blocked.drain(..).collect())
    }

    pub async fn is_unix_socket_allowed(&self, _path: &str) -> Result<bool> {
        Ok(false)
    }

    pub async fn method_allowed(&self, method: &str) -> Result<bool> {
        Ok(self.current_cfg().await?.network.mode.allows_method(method))
    }

    pub async fn allow_upstream_proxy(&self) -> Result<bool> {
        Ok(self.current_cfg().await?.network.allow_upstream_proxy)
    }

    pub async fn allow_local_binding(&self) -> Result<bool> {
        Ok(self.current_cfg().await?.network.allow_local_binding)
    }

    pub async fn network_mode(&self) -> Result<NetworkMode> {
        Ok(self.current_cfg().await?.network.mode)
    }

    pub async fn set_network_mode(&self, mode: NetworkMode) -> Result<()> {
        self.state.write().unwrap().config.network.mode = mode;
        Ok(())
    }

    pub async fn host_has_mitm_hooks(&self, _host: &str) -> Result<bool> {
        Ok(false)
    }

    pub async fn add_allowed_domain(&self, host: &str) -> Result<()> {
        self.state
            .write()
            .unwrap()
            .config
            .network
            .upsert_domain_permission(
                host.to_string(),
                NetworkDomainPermission::Allow,
                normalize_host,
            );
        Ok(())
    }

    pub async fn add_denied_domain(&self, host: &str) -> Result<()> {
        self.state
            .write()
            .unwrap()
            .config
            .network
            .upsert_domain_permission(
                host.to_string(),
                NetworkDomainPermission::Deny,
                normalize_host,
            );
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct NetworkProxyBuilder {
    state: Option<Arc<NetworkProxyState>>,
}

impl NetworkProxyBuilder {
    pub fn state(mut self, state: Arc<NetworkProxyState>) -> Self {
        self.state = Some(state);
        self
    }

    pub fn http_addr(self, _addr: SocketAddr) -> Self {
        self
    }

    pub fn socks_addr(self, _addr: SocketAddr) -> Self {
        self
    }

    pub fn managed_by_codex(self, _managed_by_codex: bool) -> Self {
        self
    }

    pub fn policy_decider<D>(self, _decider: D) -> Self
    where
        D: NetworkPolicyDecider,
    {
        self
    }

    pub fn policy_decider_arc(self, _decider: Arc<dyn NetworkPolicyDecider>) -> Self {
        self
    }

    pub fn blocked_request_observer<O>(self, _observer: O) -> Self
    where
        O: BlockedRequestObserver,
    {
        self
    }

    pub fn blocked_request_observer_arc(self, _observer: Arc<dyn BlockedRequestObserver>) -> Self {
        self
    }

    pub async fn build(self) -> Result<NetworkProxy> {
        Err(anyhow!(
            "Codex network proxy daemon is not available on wasm32; use a host fetch/Tailscale shim"
        ))
    }
}

#[derive(Clone)]
pub struct NetworkProxy {
    state: Arc<NetworkProxyState>,
}

impl std::fmt::Debug for NetworkProxy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NetworkProxy")
            .finish_non_exhaustive()
    }
}

impl NetworkProxy {
    pub fn builder() -> NetworkProxyBuilder {
        NetworkProxyBuilder::default()
    }

    pub fn http_addr(&self) -> SocketAddr {
        "127.0.0.1:0".parse().unwrap()
    }

    pub fn socks_addr(&self) -> SocketAddr {
        "127.0.0.1:0".parse().unwrap()
    }

    pub async fn current_cfg(&self) -> Result<NetworkProxyConfig> {
        self.state.current_cfg().await
    }

    pub async fn add_allowed_domain(&self, host: &str) -> Result<()> {
        self.state.add_allowed_domain(host).await
    }

    pub async fn add_denied_domain(&self, host: &str) -> Result<()> {
        self.state.add_denied_domain(host).await
    }

    pub fn allow_local_binding(&self) -> bool {
        false
    }

    pub fn allow_unix_sockets(&self) -> Arc<[String]> {
        Arc::from([])
    }

    pub fn dangerously_allow_all_unix_sockets(&self) -> bool {
        false
    }

    pub fn managed_mitm_ca_trust_bundle_path(&self) -> Option<AbsolutePathBuf> {
        None
    }

    pub fn apply_to_env(&self, env: &mut HashMap<String, String>) {
        env.insert(PROXY_ACTIVE_ENV_KEY.to_string(), "0".to_string());
    }

    pub async fn replace_config_state(&self, new_state: ConfigState) -> Result<()> {
        self.state.replace_config_state(new_state).await
    }

    pub async fn run(&self) -> Result<NetworkProxyHandle> {
        Err(anyhow!(
            "Codex network proxy daemon is not available on wasm32; use a host fetch/Tailscale shim"
        ))
    }
}

pub struct NetworkProxyHandle {}

impl NetworkProxyHandle {
    pub async fn wait(self) -> Result<()> {
        Ok(())
    }

    pub async fn shutdown(self) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkProtocol {
    Http,
    HttpsConnect,
    Socks5Tcp,
    Socks5Udp,
}

impl NetworkProtocol {
    pub const fn as_policy_protocol(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::HttpsConnect => "https_connect",
            Self::Socks5Tcp => "socks5_tcp",
            Self::Socks5Udp => "socks5_udp",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NetworkPolicyDecision {
    Deny,
    Ask,
}

impl NetworkPolicyDecision {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deny => "deny",
            Self::Ask => "ask",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkDecisionSource {
    BaselinePolicy,
    ModeGuard,
    ProxyState,
    Decider,
}

impl NetworkDecisionSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BaselinePolicy => "baseline_policy",
            Self::ModeGuard => "mode_guard",
            Self::ProxyState => "proxy_state",
            Self::Decider => "decider",
        }
    }
}

#[derive(Clone, Debug)]
pub struct NetworkPolicyRequest {
    pub protocol: NetworkProtocol,
    pub host: String,
    pub port: u16,
    pub client_addr: Option<String>,
    pub method: Option<String>,
    pub command: Option<String>,
    pub exec_policy_hint: Option<String>,
}

pub struct NetworkPolicyRequestArgs {
    pub protocol: NetworkProtocol,
    pub host: String,
    pub port: u16,
    pub client_addr: Option<String>,
    pub method: Option<String>,
    pub command: Option<String>,
    pub exec_policy_hint: Option<String>,
}

impl NetworkPolicyRequest {
    pub fn new(args: NetworkPolicyRequestArgs) -> Self {
        Self {
            protocol: args.protocol,
            host: args.host,
            port: args.port,
            client_addr: args.client_addr,
            method: args.method,
            command: args.command,
            exec_policy_hint: args.exec_policy_hint,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkDecision {
    Allow,
    Deny {
        reason: String,
        source: NetworkDecisionSource,
        decision: NetworkPolicyDecision,
    },
}

impl NetworkDecision {
    pub fn deny(reason: impl Into<String>) -> Self {
        Self::deny_with_source(reason, NetworkDecisionSource::Decider)
    }

    pub fn ask(reason: impl Into<String>) -> Self {
        Self::ask_with_source(reason, NetworkDecisionSource::Decider)
    }

    pub fn deny_with_source(reason: impl Into<String>, source: NetworkDecisionSource) -> Self {
        Self::Deny {
            reason: reason.into(),
            source,
            decision: NetworkPolicyDecision::Deny,
        }
    }

    pub fn ask_with_source(reason: impl Into<String>, source: NetworkDecisionSource) -> Self {
        Self::Deny {
            reason: reason.into(),
            source,
            decision: NetworkPolicyDecision::Ask,
        }
    }
}

#[async_trait(?Send)]
pub trait NetworkPolicyDecider: 'static {
    async fn decide(&self, req: NetworkPolicyRequest) -> NetworkDecision;
}

#[async_trait(?Send)]
impl<D: NetworkPolicyDecider + ?Sized> NetworkPolicyDecider for Arc<D> {
    async fn decide(&self, req: NetworkPolicyRequest) -> NetworkDecision {
        (**self).decide(req).await
    }
}

#[async_trait(?Send)]
impl<F, Fut> NetworkPolicyDecider for F
where
    F: Fn(NetworkPolicyRequest) -> Fut + 'static,
    Fut: Future<Output = NetworkDecision>,
{
    async fn decide(&self, req: NetworkPolicyRequest) -> NetworkDecision {
        (self)(req).await
    }
}

pub fn is_managed_mitm_ca_trust_bundle_path(_path: &str) -> bool {
    false
}

pub fn proxy_url_env_value<'a>(
    env: &'a HashMap<String, String>,
    canonical_key: &str,
) -> Option<&'a str> {
    if let Some(value) = env.get(canonical_key) {
        return Some(value.as_str());
    }
    let lower_key = canonical_key.to_ascii_lowercase();
    env.get(lower_key.as_str()).map(String::as_str)
}

pub fn has_proxy_url_env_vars(env: &HashMap<String, String>) -> bool {
    PROXY_URL_ENV_KEYS
        .iter()
        .any(|key| proxy_url_env_value(env, key).is_some_and(|value| !value.trim().is_empty()))
}

pub fn host_and_port_from_network_addr(value: &str, default_port: u16) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return format!("127.0.0.1:{default_port}");
    }
    match trimmed.parse::<url::Url>() {
        Ok(url) => match (url.host_str(), url.port_or_known_default()) {
            (Some(host), Some(port)) => format_host_port(host, port),
            (Some(host), None) => format_host_port(host, default_port),
            _ => format!("127.0.0.1:{default_port}"),
        },
        Err(_) if trimmed.contains(':') => trimmed.to_string(),
        Err(_) => format!("{trimmed}:{default_port}"),
    }
}

fn format_host_port(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

pub fn normalize_host(host: &str) -> String {
    let mut host = host.trim().to_ascii_lowercase();
    if let Some(stripped) = host
        .strip_prefix('[')
        .and_then(|value| value.split(']').next())
    {
        host = stripped.replace("%25", "%");
    } else if let Some((without_port, port)) = host.rsplit_once(':')
        && !without_port.contains(':')
        && port.parse::<u16>().is_ok()
    {
        host = without_port.to_string();
    }
    host.strip_suffix('.').unwrap_or(&host).to_string()
}
