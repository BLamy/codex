#![deny(clippy::print_stdout, clippy::print_stderr)]

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod attribution;
#[cfg(not(target_arch = "wasm32"))]
mod certs;
#[cfg(not(target_arch = "wasm32"))]
mod config;
#[cfg(not(target_arch = "wasm32"))]
mod connect_policy;
#[cfg(not(target_arch = "wasm32"))]
mod credential_broker;
#[cfg(not(target_arch = "wasm32"))]
mod http_proxy;
#[cfg(not(target_arch = "wasm32"))]
mod mitm;
#[cfg(not(target_arch = "wasm32"))]
mod mitm_hook;
#[cfg(not(target_arch = "wasm32"))]
mod native_certs;
#[cfg(not(target_arch = "wasm32"))]
mod network_policy;
#[cfg(not(target_arch = "wasm32"))]
mod policy;
#[cfg(not(target_arch = "wasm32"))]
mod proxy;
#[cfg(not(target_arch = "wasm32"))]
mod reasons;
mod remote_config;
#[cfg(not(target_arch = "wasm32"))]
mod responses;
#[cfg(not(target_arch = "wasm32"))]
mod runtime;
#[cfg(not(target_arch = "wasm32"))]
mod socks5;
#[cfg(not(target_arch = "wasm32"))]
mod state;
#[cfg(not(target_arch = "wasm32"))]
mod upstream;

#[cfg(not(target_arch = "wasm32"))]
pub use attribution::PROXY_ATTRIBUTION_TOKEN_ENV_KEY;
#[cfg(not(target_arch = "wasm32"))]
pub use attribution::write_attribution_frame;
#[cfg(not(target_arch = "wasm32"))]
pub use certs::CUSTOM_CA_ENV_KEYS;
#[cfg(not(target_arch = "wasm32"))]
pub use certs::is_managed_mitm_ca_trust_bundle_path;
#[cfg(not(target_arch = "wasm32"))]
pub use config::NetworkDomainPermission;
#[cfg(not(target_arch = "wasm32"))]
pub use config::NetworkDomainPermissionEntry;
#[cfg(not(target_arch = "wasm32"))]
pub use config::NetworkDomainPermissions;
#[cfg(not(target_arch = "wasm32"))]
pub use config::NetworkMode;
#[cfg(not(target_arch = "wasm32"))]
pub use config::NetworkProxyConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use config::NetworkUnixSocketPermission;
#[cfg(not(target_arch = "wasm32"))]
pub use config::NetworkUnixSocketPermissions;
#[cfg(not(target_arch = "wasm32"))]
pub use config::host_and_port_from_network_addr;
#[cfg(not(target_arch = "wasm32"))]
pub use credential_broker::CREDENTIAL_BROKER_ACTIVE_ENV_KEY;
#[cfg(not(target_arch = "wasm32"))]
pub use credential_broker::brokered_credential_dummy_env_keys;
#[cfg(not(target_arch = "wasm32"))]
pub use credential_broker::brokered_credential_env_keys;
#[cfg(not(target_arch = "wasm32"))]
pub use mitm_hook::InjectedHeaderConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use mitm_hook::MitmHookActionsConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use mitm_hook::MitmHookBodyConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use mitm_hook::MitmHookConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use mitm_hook::MitmHookMatchConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use network_policy::NetworkDecision;
#[cfg(not(target_arch = "wasm32"))]
pub use network_policy::NetworkDecisionSource;
#[cfg(not(target_arch = "wasm32"))]
pub use network_policy::NetworkPolicyDecider;
#[cfg(not(target_arch = "wasm32"))]
pub use network_policy::NetworkPolicyDeciderFuture;
#[cfg(not(target_arch = "wasm32"))]
pub use network_policy::NetworkPolicyDecision;
#[cfg(not(target_arch = "wasm32"))]
pub use network_policy::NetworkPolicyRequest;
#[cfg(not(target_arch = "wasm32"))]
pub use network_policy::NetworkPolicyRequestArgs;
#[cfg(not(target_arch = "wasm32"))]
pub use network_policy::NetworkProtocol;
#[cfg(not(target_arch = "wasm32"))]
pub use policy::normalize_host;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::ALL_PROXY_ENV_KEYS;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::ALLOW_LOCAL_BINDING_ENV_KEY;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::Args;
#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
pub use proxy::CODEX_PROXY_GIT_SSH_COMMAND_MARKER;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::DEFAULT_NO_PROXY_VALUE;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::ManagedNetworkSandboxContext;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::NO_PROXY_ENV_KEYS;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::NetworkProxy;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::NetworkProxyBuilder;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::NetworkProxyHandle;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::PROXY_ACTIVE_ENV_KEY;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::PROXY_ENV_KEYS;
#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
pub use proxy::PROXY_GIT_SSH_COMMAND_ENV_KEY;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::PROXY_URL_ENV_KEYS;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::PreparedManagedNetwork;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::has_proxy_url_env_vars;
#[cfg(not(target_arch = "wasm32"))]
pub use proxy::proxy_url_env_value;
pub use remote_config::RemoteNetworkProxyConfig;
pub use remote_config::RemoteNetworkProxyLaunchConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::BlockedRequest;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::BlockedRequestArgs;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::BlockedRequestObserver;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::BlockedRequestObserverFuture;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::ConfigReloader;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::ConfigReloaderFuture;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::ConfigState;
#[cfg(not(target_arch = "wasm32"))]
pub use runtime::NetworkProxyState;
#[cfg(not(target_arch = "wasm32"))]
pub use state::NetworkProxyAuditMetadata;
#[cfg(not(target_arch = "wasm32"))]
pub use state::NetworkProxyConstraintError;
#[cfg(not(target_arch = "wasm32"))]
pub use state::NetworkProxyConstraints;
#[cfg(not(target_arch = "wasm32"))]
pub use state::PartialNetworkProxyConfig;
#[cfg(not(target_arch = "wasm32"))]
pub use state::build_config_state;
#[cfg(not(target_arch = "wasm32"))]
pub use state::validate_policy_against_constraints;
