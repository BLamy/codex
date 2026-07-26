//! Browser boundary for Agent Identity.
//!
//! Agent key generation, JWKS verification, and task registration require
//! native crypto/storage facilities. Browser hosts can still supply API-key or
//! ChatGPT token auth through `ExternalAuth`.

use std::future::Future;
use std::sync::Arc;

use codex_protocol::account::PlanType as AccountPlanType;
use codex_protocol::protocol::SessionSource;
use thiserror::Error;

use crate::outbound_proxy::AuthRouteConfig;

use super::storage::AgentIdentityAuthRecord;

pub(super) const MAX_AGENT_IDENTITY_BOOTSTRAP_ATTEMPTS: usize = 3;

pub(super) fn agent_identity_authapi_base_url(
    chatgpt_base_url: Option<&str>,
) -> std::io::Result<String> {
    let base_url = chatgpt_base_url.unwrap_or("https://chatgpt.com/backend-api");
    match base_url.trim_end_matches('/') {
        "https://chatgpt.com"
        | "https://chatgpt.com/backend-api"
        | "https://chatgpt.com/codex"
        | "https://chatgpt.com/backend-api/codex"
        | "https://chat.openai.com"
        | "https://chat.openai.com/backend-api"
        | "https://chat.openai.com/codex"
        | "https://chat.openai.com/backend-api/codex" => {
            Ok("https://auth.openai.com/api/accounts".to_string())
        }
        "https://chatgpt-staging.com"
        | "https://chatgpt-staging.com/backend-api"
        | "https://chatgpt-staging.com/codex"
        | "https://chatgpt-staging.com/backend-api/codex" => {
            Ok("https://auth.api.openai.org/api/accounts".to_string())
        }
        _ => Err(std::io::Error::other(
            "Agent Identity only supports production and staging ChatGPT environments",
        )),
    }
}

pub(super) fn require_agent_identity_authapi_base_url(
    agent_identity_authapi_base_url: Option<&str>,
) -> std::io::Result<&str> {
    agent_identity_authapi_base_url.ok_or_else(|| {
        std::io::Error::other(
            "Agent Identity only supports production and staging ChatGPT environments",
        )
    })
}

#[derive(Clone, Debug, Error)]
pub enum AgentIdentityAuthError {
    #[error(
        "agent identity bootstrap unavailable after {attempts} attempts during {operation}: {message}"
    )]
    BootstrapUnavailable {
        operation: &'static str,
        attempts: usize,
        message: String,
    },
}

impl AgentIdentityAuthError {
    pub(super) fn bootstrap_unavailable(error: &std::io::Error) -> Option<&Self> {
        error
            .get_ref()
            .and_then(|source| source.downcast_ref::<Self>())
    }
}

#[derive(Debug, Error)]
#[error("retryable agent identity registration failure: {message}")]
pub(super) struct RetryableAgentIdentityRegistrationError {
    message: String,
}

impl RetryableAgentIdentityRegistrationError {
    #[allow(dead_code)]
    pub(super) fn new(message: String) -> Self {
        Self { message }
    }
}

#[derive(Clone, Debug)]
pub struct AgentIdentityAuth {
    record: Arc<AgentIdentityAuthRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ManagedChatGptAgentIdentityBinding {
    pub(super) account_id: String,
    pub(super) chatgpt_user_id: String,
    pub(super) email: Option<String>,
    pub(super) plan_type: AccountPlanType,
    pub(super) chatgpt_account_is_fedramp: bool,
    pub(super) access_token: String,
}

impl AgentIdentityAuth {
    pub async fn from_record(
        record: AgentIdentityAuthRecord,
        _agent_identity_authapi_base_url: &str,
        _auth_route_config: Option<&AuthRouteConfig>,
    ) -> std::io::Result<Self> {
        if record_needs_task_registration(&record) {
            return Err(std::io::Error::other(
                "agent identity task registration is unavailable in the browser runtime",
            ));
        }
        Ok(Self {
            record: Arc::new(record),
        })
    }

    pub async fn from_jwt(
        _jwt: &str,
        _chatgpt_base_url: &str,
        _agent_identity_authapi_base_url: &str,
        _auth_route_config: Option<&AuthRouteConfig>,
    ) -> std::io::Result<Self> {
        Err(std::io::Error::other(
            "agent identity JWT verification is unavailable in the browser runtime",
        ))
    }

    pub fn record(&self) -> &AgentIdentityAuthRecord {
        self.record.as_ref()
    }

    pub fn run_task_id(&self) -> &str {
        self.record.task_id.as_deref().unwrap_or_default()
    }

    pub fn account_id(&self) -> &str {
        &self.record.account_id
    }

    pub fn chatgpt_user_id(&self) -> &str {
        &self.record.chatgpt_user_id
    }

    pub fn email(&self) -> Option<&str> {
        self.record.email.as_deref()
    }

    pub fn plan_type(&self) -> AccountPlanType {
        self.record.plan_type
    }

    pub fn is_fedramp_account(&self) -> bool {
        self.record.chatgpt_account_is_fedramp
    }
}

pub(super) async fn register_managed_chatgpt_agent_identity(
    _binding: ManagedChatGptAgentIdentityBinding,
    _agent_identity_authapi_base_url: &str,
    _session_source: SessionSource,
    _auth_route_config: Option<&AuthRouteConfig>,
) -> std::io::Result<AgentIdentityAuth> {
    Err(std::io::Error::other(
        "managed agent identity registration is unavailable in the browser runtime",
    ))
}

pub(super) async fn verified_record_from_jwt(
    _jwt: &str,
    _chatgpt_base_url: &str,
    _auth_route_config: Option<&AuthRouteConfig>,
) -> std::io::Result<AgentIdentityAuthRecord> {
    Err(std::io::Error::other(
        "agent identity JWT verification is unavailable in the browser runtime",
    ))
}

pub(super) fn record_needs_task_registration(record: &AgentIdentityAuthRecord) -> bool {
    record
        .task_id
        .as_deref()
        .is_none_or(|task_id| task_id.trim().is_empty())
}

pub(super) fn record_matches_managed_chatgpt_binding(
    record: &AgentIdentityAuthRecord,
    binding: &ManagedChatGptAgentIdentityBinding,
) -> bool {
    record.account_id == binding.account_id
        && record.chatgpt_user_id == binding.chatgpt_user_id
        && !record.agent_private_key.trim().is_empty()
}

pub(super) fn classify_bootstrap_error(
    operation: &'static str,
    err: std::io::Error,
) -> std::io::Error {
    if is_retryable_io_registration_error(&err) {
        std::io::Error::other(AgentIdentityAuthError::BootstrapUnavailable {
            operation,
            attempts: MAX_AGENT_IDENTITY_BOOTSTRAP_ATTEMPTS,
            message: err.to_string(),
        })
    } else {
        err
    }
}

pub(super) fn is_retryable_io_registration_error(err: &std::io::Error) -> bool {
    err.get_ref().is_some_and(
        <dyn std::error::Error + Send + Sync + 'static>::is::<
            RetryableAgentIdentityRegistrationError,
        >,
    )
}

#[allow(dead_code)]
pub(super) async fn retry_registration<T, F, Fut>(mut operation: F) -> std::io::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = std::io::Result<T>>,
{
    operation().await
}
