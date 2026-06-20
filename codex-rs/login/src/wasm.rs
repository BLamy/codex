use std::env;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use codex_app_server_protocol::AuthMode;
use codex_app_server_protocol::AuthMode as ApiAuthMode;
use codex_config::types::AuthCredentialsStoreMode;
use codex_protocol::account::PlanType as AccountPlanType;
use codex_protocol::auth::PlanType as InternalPlanType;
use codex_protocol::auth::RefreshTokenFailedError;
use codex_protocol::auth::RefreshTokenFailedReason;
use codex_protocol::config_types::ForcedLoginMethod;
use codex_protocol::config_types::ModelProviderAuthInfo;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::watch;

use crate::token_data::TokenData;
use crate::token_data::parse_chatgpt_jwt_claims;

pub const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
pub const OPENAI_API_KEY_ENV_VAR: &str = "OPENAI_API_KEY";
pub const CODEX_API_KEY_ENV_VAR: &str = "CODEX_API_KEY";
pub const CODEX_ACCESS_TOKEN_ENV_VAR: &str = "CODEX_ACCESS_TOKEN";
pub const REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR: &str = "CODEX_REFRESH_TOKEN_URL_OVERRIDE";
pub const REVOKE_TOKEN_URL_OVERRIDE_ENV_VAR: &str = "CODEX_REVOKE_TOKEN_URL_OVERRIDE";

pub type BuildLoginHttpClientError = codex_client::BuildCustomCaTransportError;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct AuthDotJson {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<AuthMode>,
    #[serde(rename = "OPENAI_API_KEY")]
    pub openai_api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokenData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_identity: Option<String>,
}

impl AuthDotJson {
    pub fn resolved_mode(&self) -> AuthMode {
        if let Some(mode) = self.auth_mode {
            return mode;
        }
        if self.openai_api_key.is_some() {
            return AuthMode::ApiKey;
        }
        if self.agent_identity.is_some() {
            return AuthMode::AgentIdentity;
        }
        AuthMode::Chatgpt
    }

    pub fn storage_mode(&self, configured: AuthCredentialsStoreMode) -> AuthCredentialsStoreMode {
        if self.auth_mode == Some(AuthMode::ChatgptAuthTokens) {
            AuthCredentialsStoreMode::Ephemeral
        } else {
            configured
        }
    }

    pub fn from_external_access_token(
        access_token: &str,
        chatgpt_account_id: &str,
        chatgpt_plan_type: Option<&str>,
    ) -> std::io::Result<Self> {
        let plan_type = chatgpt_plan_type.map(|plan| InternalPlanType::Unknown(plan.to_string()));
        Ok(Self {
            auth_mode: Some(AuthMode::ChatgptAuthTokens),
            openai_api_key: None,
            tokens: Some(TokenData {
                id_token: crate::token_data::IdTokenInfo {
                    email: None,
                    chatgpt_plan_type: plan_type,
                    chatgpt_user_id: None,
                    chatgpt_account_id: Some(chatgpt_account_id.to_string()),
                    chatgpt_account_is_fedramp: false,
                    raw_jwt: String::new(),
                },
                access_token: access_token.to_string(),
                refresh_token: String::new(),
                account_id: Some(chatgpt_account_id.to_string()),
            }),
            last_refresh: Some(Utc::now()),
            agent_identity: None,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct AgentIdentityAuthRecord {
    pub agent_runtime_id: String,
    pub agent_private_key: String,
    pub account_id: String,
    pub chatgpt_user_id: String,
    pub email: String,
    pub plan_type: AccountPlanType,
    pub chatgpt_account_is_fedramp: bool,
}

#[derive(Clone, Debug)]
pub struct AgentIdentityAuth {
    record: AgentIdentityAuthRecord,
    process_task_id: String,
}

impl AgentIdentityAuth {
    pub async fn load(record: AgentIdentityAuthRecord) -> std::io::Result<Self> {
        Ok(Self {
            record,
            process_task_id: "browser-agent-task".to_string(),
        })
    }

    pub fn record(&self) -> &AgentIdentityAuthRecord {
        &self.record
    }

    pub fn process_task_id(&self) -> &str {
        &self.process_task_id
    }

    pub fn account_id(&self) -> &str {
        &self.record.account_id
    }

    pub fn chatgpt_user_id(&self) -> &str {
        &self.record.chatgpt_user_id
    }

    pub fn email(&self) -> &str {
        &self.record.email
    }

    pub fn plan_type(&self) -> AccountPlanType {
        self.record.plan_type
    }

    pub fn is_fedramp_account(&self) -> bool {
        self.record.chatgpt_account_is_fedramp
    }
}

#[derive(Debug, Clone)]
pub enum CodexAuth {
    ApiKey(ApiKeyAuth),
    Chatgpt(ChatgptAuth),
    ChatgptAuthTokens(ChatgptAuthTokens),
    AgentIdentity(AgentIdentityAuth),
}

impl PartialEq for CodexAuth {
    fn eq(&self, other: &Self) -> bool {
        self.api_auth_mode() == other.api_auth_mode()
    }
}

#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    api_key: String,
}

#[derive(Debug, Clone)]
pub struct ChatgptAuth {
    auth_dot_json: AuthDotJson,
}

#[derive(Debug, Clone)]
pub struct ChatgptAuthTokens {
    auth_dot_json: AuthDotJson,
}

impl CodexAuth {
    pub async fn from_auth_storage(
        codex_home: &Path,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
        _chatgpt_base_url: Option<&str>,
    ) -> std::io::Result<Option<Self>> {
        load_auth(codex_home, false, auth_credentials_store_mode).await
    }

    pub async fn from_agent_identity_jwt(
        _jwt: &str,
        _chatgpt_base_url: Option<&str>,
    ) -> std::io::Result<Self> {
        Err(std::io::Error::other(
            "agent identity JWT verification needs a browser auth host shim",
        ))
    }

    pub fn auth_mode(&self) -> AuthMode {
        match self {
            Self::ApiKey(_) => AuthMode::ApiKey,
            Self::Chatgpt(_) => AuthMode::Chatgpt,
            Self::ChatgptAuthTokens(_) => AuthMode::ChatgptAuthTokens,
            Self::AgentIdentity(_) => AuthMode::AgentIdentity,
        }
    }

    pub fn api_auth_mode(&self) -> ApiAuthMode {
        self.auth_mode()
    }

    pub fn is_api_key_auth(&self) -> bool {
        self.auth_mode() == AuthMode::ApiKey
    }

    pub fn is_chatgpt_auth(&self) -> bool {
        matches!(self, Self::Chatgpt(_) | Self::ChatgptAuthTokens(_))
    }

    pub fn uses_codex_backend(&self) -> bool {
        matches!(
            self,
            Self::Chatgpt(_) | Self::ChatgptAuthTokens(_) | Self::AgentIdentity(_)
        )
    }

    pub fn is_external_chatgpt_tokens(&self) -> bool {
        matches!(self, Self::ChatgptAuthTokens(_))
    }

    pub fn api_key(&self) -> Option<&str> {
        match self {
            Self::ApiKey(auth) => Some(auth.api_key.as_str()),
            _ => None,
        }
    }

    pub fn get_token_data(&self) -> Result<TokenData, std::io::Error> {
        self.get_current_token_data()
            .ok_or_else(|| std::io::Error::other("Token data is not available."))
    }

    pub fn get_token(&self) -> Result<String, std::io::Error> {
        match self {
            Self::ApiKey(auth) => Ok(auth.api_key.clone()),
            Self::Chatgpt(_) | Self::ChatgptAuthTokens(_) => {
                Ok(self.get_token_data()?.access_token)
            }
            Self::AgentIdentity(_) => Err(std::io::Error::other(
                "agent identity auth does not expose a bearer token",
            )),
        }
    }

    pub fn get_account_id(&self) -> Option<String> {
        match self {
            Self::AgentIdentity(auth) => Some(auth.account_id().to_string()),
            _ => self.get_current_token_data().and_then(|t| t.account_id),
        }
    }

    pub fn is_fedramp_account(&self) -> bool {
        match self {
            Self::AgentIdentity(auth) => auth.is_fedramp_account(),
            _ => self
                .get_current_token_data()
                .is_some_and(|t| t.id_token.is_fedramp_account()),
        }
    }

    pub fn get_account_email(&self) -> Option<String> {
        match self {
            Self::AgentIdentity(auth) => Some(auth.email().to_string()),
            _ => self.get_current_token_data().and_then(|t| t.id_token.email),
        }
    }

    pub fn get_chatgpt_user_id(&self) -> Option<String> {
        match self {
            Self::AgentIdentity(auth) => Some(auth.chatgpt_user_id().to_string()),
            _ => self
                .get_current_token_data()
                .and_then(|t| t.id_token.chatgpt_user_id),
        }
    }

    pub fn account_plan_type(&self) -> Option<AccountPlanType> {
        if let Self::AgentIdentity(auth) = self {
            return Some(auth.plan_type());
        }

        self.get_current_token_data().map(|t| {
            t.id_token
                .chatgpt_plan_type
                .map(AccountPlanType::from)
                .unwrap_or(AccountPlanType::Unknown)
        })
    }

    pub fn is_workspace_account(&self) -> bool {
        self.account_plan_type()
            .is_some_and(AccountPlanType::is_workspace_account)
    }

    pub fn get_current_auth_json(&self) -> Option<AuthDotJson> {
        match self {
            Self::Chatgpt(auth) => Some(auth.auth_dot_json.clone()),
            Self::ChatgptAuthTokens(auth) => Some(auth.auth_dot_json.clone()),
            Self::ApiKey(_) | Self::AgentIdentity(_) => None,
        }
    }

    fn get_current_token_data(&self) -> Option<TokenData> {
        self.get_current_auth_json().and_then(|auth| auth.tokens)
    }

    pub fn create_dummy_chatgpt_auth_for_testing() -> Self {
        let auth_dot_json = AuthDotJson {
            auth_mode: Some(AuthMode::Chatgpt),
            openai_api_key: None,
            tokens: Some(TokenData {
                id_token: Default::default(),
                access_token: "Access Token".to_string(),
                refresh_token: "test".to_string(),
                account_id: Some("account_id".to_string()),
            }),
            last_refresh: Some(Utc::now()),
            agent_identity: None,
        };
        Self::Chatgpt(ChatgptAuth { auth_dot_json })
    }

    pub fn from_api_key(api_key: &str) -> Self {
        Self::ApiKey(ApiKeyAuth {
            api_key: api_key.to_owned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalAuthTokens {
    pub access_token: String,
    pub chatgpt_metadata: Option<ExternalAuthChatgptMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalAuthChatgptMetadata {
    pub account_id: String,
    pub plan_type: Option<String>,
}

impl ExternalAuthTokens {
    pub fn access_token_only(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            chatgpt_metadata: None,
        }
    }

    pub fn chatgpt(
        access_token: impl Into<String>,
        chatgpt_account_id: impl Into<String>,
        chatgpt_plan_type: Option<String>,
    ) -> Self {
        Self {
            access_token: access_token.into(),
            chatgpt_metadata: Some(ExternalAuthChatgptMetadata {
                account_id: chatgpt_account_id.into(),
                plan_type: chatgpt_plan_type,
            }),
        }
    }

    pub fn chatgpt_metadata(&self) -> Option<&ExternalAuthChatgptMetadata> {
        self.chatgpt_metadata.as_ref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalAuthRefreshReason {
    Unauthorized,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalAuthRefreshContext {
    pub reason: ExternalAuthRefreshReason,
    pub previous_account_id: Option<String>,
}

#[async_trait]
pub trait ExternalAuth: Send + Sync {
    fn auth_mode(&self) -> AuthMode;

    async fn resolve(&self) -> std::io::Result<Option<ExternalAuthTokens>> {
        Ok(None)
    }

    async fn refresh(
        &self,
        _context: ExternalAuthRefreshContext,
    ) -> std::io::Result<ExternalAuthTokens> {
        Err(std::io::Error::other(
            "external auth refresh needs a browser auth host shim",
        ))
    }
}

#[derive(Debug, Error)]
pub enum RefreshTokenError {
    #[error("{0}")]
    Permanent(#[from] RefreshTokenFailedError),
    #[error(transparent)]
    Transient(#[from] std::io::Error),
}

impl RefreshTokenError {
    pub fn failed_reason(&self) -> Option<RefreshTokenFailedReason> {
        match self {
            Self::Permanent(error) => Some(error.reason),
            Self::Transient(_) => None,
        }
    }
}

impl From<RefreshTokenError> for std::io::Error {
    fn from(err: RefreshTokenError) -> Self {
        match err {
            RefreshTokenError::Permanent(failed) => std::io::Error::other(failed),
            RefreshTokenError::Transient(inner) => inner,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
    pub codex_home: PathBuf,
    pub auth_credentials_store_mode: AuthCredentialsStoreMode,
    pub forced_login_method: Option<ForcedLoginMethod>,
    pub chatgpt_base_url: Option<String>,
    pub forced_chatgpt_workspace_id: Option<Vec<String>>,
}

pub trait AuthManagerConfig {
    fn codex_home(&self) -> PathBuf;
    fn cli_auth_credentials_store_mode(&self) -> AuthCredentialsStoreMode;
    fn forced_chatgpt_workspace_id(&self) -> Option<Vec<String>>;
    fn chatgpt_base_url(&self) -> String;
}

pub struct AuthManager {
    codex_home: PathBuf,
    inner: RwLock<Option<CodexAuth>>,
    auth_change_tx: watch::Sender<u64>,
    enable_codex_api_key_env: bool,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    forced_chatgpt_workspace_id: RwLock<Option<Vec<String>>>,
    chatgpt_base_url: Option<String>,
    external_auth: RwLock<Option<Arc<dyn ExternalAuth>>>,
}

impl Debug for AuthManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthManager")
            .field("codex_home", &self.codex_home)
            .field("inner", &self.inner)
            .field("enable_codex_api_key_env", &self.enable_codex_api_key_env)
            .field(
                "auth_credentials_store_mode",
                &self.auth_credentials_store_mode,
            )
            .field(
                "forced_chatgpt_workspace_id",
                &self.forced_chatgpt_workspace_id,
            )
            .field("chatgpt_base_url", &self.chatgpt_base_url)
            .field("has_external_auth", &self.has_external_auth())
            .finish_non_exhaustive()
    }
}

impl AuthManager {
    pub async fn new(
        codex_home: PathBuf,
        enable_codex_api_key_env: bool,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
        chatgpt_base_url: Option<String>,
    ) -> Self {
        let auth = load_auth(
            &codex_home,
            enable_codex_api_key_env,
            auth_credentials_store_mode,
        )
        .await
        .ok()
        .flatten();
        let (auth_change_tx, _auth_change_rx) = watch::channel(0);
        Self {
            codex_home,
            inner: RwLock::new(auth),
            auth_change_tx,
            enable_codex_api_key_env,
            auth_credentials_store_mode,
            forced_chatgpt_workspace_id: RwLock::new(None),
            chatgpt_base_url,
            external_auth: RwLock::new(None),
        }
    }

    pub fn from_auth_for_testing(auth: CodexAuth) -> Arc<Self> {
        Self::from_auth_for_testing_with_home(auth, PathBuf::from("non-existent"))
    }

    pub fn from_auth_for_testing_with_home(auth: CodexAuth, codex_home: PathBuf) -> Arc<Self> {
        let (auth_change_tx, _auth_change_rx) = watch::channel(0);
        Arc::new(Self {
            codex_home,
            inner: RwLock::new(Some(auth)),
            auth_change_tx,
            enable_codex_api_key_env: false,
            auth_credentials_store_mode: AuthCredentialsStoreMode::File,
            forced_chatgpt_workspace_id: RwLock::new(None),
            chatgpt_base_url: None,
            external_auth: RwLock::new(None),
        })
    }

    pub fn external_bearer_only(_config: ModelProviderAuthInfo) -> Arc<Self> {
        let (auth_change_tx, _auth_change_rx) = watch::channel(0);
        Arc::new(Self {
            codex_home: PathBuf::from("non-existent"),
            inner: RwLock::new(None),
            auth_change_tx,
            enable_codex_api_key_env: false,
            auth_credentials_store_mode: AuthCredentialsStoreMode::File,
            forced_chatgpt_workspace_id: RwLock::new(None),
            chatgpt_base_url: None,
            external_auth: RwLock::new(None),
        })
    }

    pub fn auth_cached(&self) -> Option<CodexAuth> {
        self.inner.read().ok().and_then(|auth| auth.clone())
    }

    pub fn auth_change_receiver(&self) -> watch::Receiver<u64> {
        self.auth_change_tx.subscribe()
    }

    pub fn refresh_failure_for_auth(&self, _auth: &CodexAuth) -> Option<RefreshTokenFailedError> {
        None
    }

    pub async fn auth(&self) -> Option<CodexAuth> {
        if let Some(auth) = self.resolve_external_auth().await {
            return Some(auth);
        }
        self.auth_cached()
    }

    pub async fn reload(&self) -> bool {
        match load_auth(
            &self.codex_home,
            self.enable_codex_api_key_env,
            self.auth_credentials_store_mode,
        )
        .await
        {
            Ok(auth) => self.set_cached_auth(auth),
            Err(_) => false,
        }
    }

    fn set_cached_auth(&self, new_auth: Option<CodexAuth>) -> bool {
        let Ok(mut guard) = self.inner.write() else {
            return false;
        };
        let changed = *guard != new_auth;
        *guard = new_auth;
        if changed {
            self.auth_change_tx.send_modify(|revision| *revision += 1);
        }
        changed
    }

    pub fn set_external_auth(&self, external_auth: Arc<dyn ExternalAuth>) {
        if let Ok(mut guard) = self.external_auth.write() {
            *guard = Some(external_auth);
        }
    }

    pub fn clear_external_auth(&self) {
        if let Ok(mut guard) = self.external_auth.write() {
            *guard = None;
        }
    }

    pub fn set_forced_chatgpt_workspace_id(&self, workspace_id: Option<Vec<String>>) {
        if let Ok(mut guard) = self.forced_chatgpt_workspace_id.write() {
            *guard = workspace_id;
        }
    }

    pub fn forced_chatgpt_workspace_id(&self) -> Option<Vec<String>> {
        self.forced_chatgpt_workspace_id
            .read()
            .ok()
            .and_then(|guard| guard.clone())
    }

    pub fn has_external_auth(&self) -> bool {
        self.external_auth
            .read()
            .ok()
            .is_some_and(|guard| guard.is_some())
    }

    pub fn is_external_chatgpt_auth_active(&self) -> bool {
        self.auth_cached()
            .as_ref()
            .is_some_and(CodexAuth::is_external_chatgpt_tokens)
    }

    pub fn codex_api_key_env_enabled(&self) -> bool {
        self.enable_codex_api_key_env
    }

    pub async fn shared(
        codex_home: PathBuf,
        enable_codex_api_key_env: bool,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
        chatgpt_base_url: Option<String>,
    ) -> Arc<Self> {
        Arc::new(
            Self::new(
                codex_home,
                enable_codex_api_key_env,
                auth_credentials_store_mode,
                chatgpt_base_url,
            )
            .await,
        )
    }

    pub async fn shared_from_config(
        config: &impl AuthManagerConfig,
        enable_codex_api_key_env: bool,
    ) -> Arc<Self> {
        let auth_manager = Self::shared(
            config.codex_home(),
            enable_codex_api_key_env,
            config.cli_auth_credentials_store_mode(),
            Some(config.chatgpt_base_url()),
        )
        .await;
        auth_manager.set_forced_chatgpt_workspace_id(config.forced_chatgpt_workspace_id());
        auth_manager
    }

    pub fn unauthorized_recovery(self: &Arc<Self>) -> UnauthorizedRecovery {
        UnauthorizedRecovery::new(Arc::clone(self))
    }

    pub async fn refresh_token(&self) -> Result<(), RefreshTokenError> {
        self.refresh_external_auth(ExternalAuthRefreshReason::Unauthorized)
            .await
    }

    pub async fn refresh_external_auth(
        &self,
        reason: ExternalAuthRefreshReason,
    ) -> Result<(), RefreshTokenError> {
        let Some(external) = self
            .external_auth
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().cloned())
        else {
            return Err(RefreshTokenError::Transient(std::io::Error::other(
                "token refresh needs a browser auth host shim",
            )));
        };
        let previous_account_id = self
            .auth_cached()
            .as_ref()
            .and_then(CodexAuth::get_account_id);
        let tokens = external
            .refresh(ExternalAuthRefreshContext {
                reason,
                previous_account_id,
            })
            .await
            .map_err(RefreshTokenError::Transient)?;
        let auth = codex_auth_from_external_tokens(external.auth_mode(), &tokens)
            .ok_or_else(|| {
                RefreshTokenError::Transient(std::io::Error::other(
                    "external auth refresh returned unusable tokens",
                ))
            })?;
        self.set_cached_auth(Some(auth));
        Ok(())
    }

    pub async fn logout(&self) -> std::io::Result<bool> {
        logout(&self.codex_home, self.auth_credentials_store_mode)
    }

    pub async fn logout_with_revoke(&self) -> std::io::Result<bool> {
        logout(&self.codex_home, self.auth_credentials_store_mode)
    }

    pub fn get_api_auth_mode(&self) -> Option<ApiAuthMode> {
        self.auth_cached().as_ref().map(CodexAuth::api_auth_mode)
    }

    pub fn auth_mode(&self) -> Option<AuthMode> {
        self.auth_cached().as_ref().map(CodexAuth::auth_mode)
    }

    pub fn current_auth_uses_codex_backend(&self) -> bool {
        self.auth_cached()
            .as_ref()
            .is_some_and(CodexAuth::uses_codex_backend)
    }

    async fn resolve_external_auth(&self) -> Option<CodexAuth> {
        let external = self
            .external_auth
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().cloned())?;
        let tokens = external.resolve().await.ok().flatten()?;
        codex_auth_from_external_tokens(external.auth_mode(), &tokens)
    }
}

fn codex_auth_from_external_tokens(
    auth_mode: AuthMode,
    tokens: &ExternalAuthTokens,
) -> Option<CodexAuth> {
    match auth_mode {
        AuthMode::ApiKey => Some(CodexAuth::from_api_key(&tokens.access_token)),
        AuthMode::Chatgpt | AuthMode::ChatgptAuthTokens => {
            let meta = tokens.chatgpt_metadata();
            let auth_dot_json = AuthDotJson::from_external_access_token(
                &tokens.access_token,
                meta.map(|m| m.account_id.as_str()).unwrap_or(""),
                meta.and_then(|m| m.plan_type.as_deref()),
            )
            .ok()?;
            Some(CodexAuth::ChatgptAuthTokens(ChatgptAuthTokens {
                auth_dot_json,
            }))
        }
        AuthMode::AgentIdentity => None,
    }
}

// Browser counterpart of the native UnauthorizedRecovery state machine. The
// only recovery source in the wasm build is the host-provided ExternalAuth
// (one refresh attempt per 401, matching the native external-auth mode).
pub struct UnauthorizedRecovery {
    auth_manager: Arc<AuthManager>,
    done: bool,
}

impl UnauthorizedRecovery {
    fn new(auth_manager: Arc<AuthManager>) -> Self {
        Self {
            auth_manager,
            done: false,
        }
    }

    pub fn has_next(&self) -> bool {
        !self.done && self.auth_manager.has_external_auth()
    }

    pub fn unavailable_reason(&self) -> &'static str {
        if !self.auth_manager.has_external_auth() {
            return "no_external_auth";
        }
        if self.done {
            return "recovery_exhausted";
        }
        "ready"
    }

    pub fn mode_name(&self) -> &'static str {
        "external"
    }

    pub fn step_name(&self) -> &'static str {
        if self.done { "done" } else { "external_refresh" }
    }

    pub async fn next(&mut self) -> Result<UnauthorizedRecoveryStepResult, RefreshTokenError> {
        if !self.has_next() {
            return Err(std::io::Error::other("No more recovery steps available.").into());
        }
        self.done = true;
        self.auth_manager
            .refresh_external_auth(ExternalAuthRefreshReason::Unauthorized)
            .await?;
        Ok(UnauthorizedRecoveryStepResult {
            auth_state_changed: Some(true),
        })
    }
}

pub struct UnauthorizedRecoveryStepResult {
    auth_state_changed: Option<bool>,
}

impl UnauthorizedRecoveryStepResult {
    pub fn auth_state_changed(&self) -> Option<bool> {
        self.auth_state_changed
    }
}

pub async fn enforce_login_restrictions(_config: &AuthConfig) -> std::io::Result<()> {
    Ok(())
}

pub fn read_openai_api_key_from_env() -> Option<String> {
    env::var(OPENAI_API_KEY_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn read_codex_api_key_from_env() -> Option<String> {
    read_non_empty_env_var(CODEX_API_KEY_ENV_VAR)
}

pub fn read_codex_access_token_from_env() -> Option<String> {
    read_non_empty_env_var(CODEX_ACCESS_TOKEN_ENV_VAR)
}

fn read_non_empty_env_var(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn login_with_api_key(
    _codex_home: &Path,
    _api_key: &str,
    _auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<()> {
    Err(std::io::Error::other(
        "saving API key auth needs a browser auth storage host shim",
    ))
}

pub async fn login_with_access_token(
    _codex_home: &Path,
    _access_token: &str,
    _auth_credentials_store_mode: AuthCredentialsStoreMode,
    _chatgpt_base_url: Option<&str>,
) -> std::io::Result<()> {
    Err(std::io::Error::other(
        "saving access token auth needs a browser auth storage host shim",
    ))
}

pub fn save_auth(
    _codex_home: &Path,
    _auth: &AuthDotJson,
    _auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<()> {
    Err(std::io::Error::other(
        "saving auth needs a browser auth storage host shim",
    ))
}

pub fn load_auth_dot_json(
    _codex_home: &Path,
    _auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<Option<AuthDotJson>> {
    Ok(auth_dot_json_from_env())
}

pub fn logout(
    _codex_home: &Path,
    _auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<bool> {
    Ok(false)
}

pub async fn logout_with_revoke(
    _codex_home: &Path,
    _auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<bool> {
    Ok(false)
}

async fn load_auth(
    codex_home: &Path,
    enable_codex_api_key_env: bool,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<Option<CodexAuth>> {
    if enable_codex_api_key_env && let Some(api_key) = read_codex_api_key_from_env() {
        return Ok(Some(CodexAuth::from_api_key(&api_key)));
    }
    if let Some(api_key) = read_openai_api_key_from_env() {
        return Ok(Some(CodexAuth::from_api_key(&api_key)));
    }
    if let Some(access_token) = read_codex_access_token_from_env() {
        let auth_dot_json = auth_dot_json_from_access_token(&access_token)?;
        return Ok(Some(CodexAuth::ChatgptAuthTokens(ChatgptAuthTokens {
            auth_dot_json,
        })));
    }
    load_auth_dot_json(codex_home, auth_credentials_store_mode)
        .map(|auth| auth.and_then(codex_auth_from_auth_dot_json))
}

fn codex_auth_from_auth_dot_json(auth_dot_json: AuthDotJson) -> Option<CodexAuth> {
    match auth_dot_json.resolved_mode() {
        AuthMode::ApiKey => auth_dot_json
            .openai_api_key
            .as_deref()
            .map(CodexAuth::from_api_key),
        AuthMode::Chatgpt => Some(CodexAuth::Chatgpt(ChatgptAuth { auth_dot_json })),
        AuthMode::ChatgptAuthTokens => Some(CodexAuth::ChatgptAuthTokens(ChatgptAuthTokens {
            auth_dot_json,
        })),
        AuthMode::AgentIdentity => None,
    }
}

fn auth_dot_json_from_env() -> Option<AuthDotJson> {
    if let Some(api_key) = read_openai_api_key_from_env().or_else(read_codex_api_key_from_env) {
        return Some(AuthDotJson {
            auth_mode: Some(AuthMode::ApiKey),
            openai_api_key: Some(api_key),
            tokens: None,
            last_refresh: None,
            agent_identity: None,
        });
    }
    read_codex_access_token_from_env()
        .and_then(|access_token| auth_dot_json_from_access_token(&access_token).ok())
}

fn auth_dot_json_from_access_token(access_token: &str) -> std::io::Result<AuthDotJson> {
    let id_token = parse_chatgpt_jwt_claims(access_token).unwrap_or_default();
    Ok(AuthDotJson {
        auth_mode: Some(AuthMode::ChatgptAuthTokens),
        openai_api_key: None,
        tokens: Some(TokenData {
            account_id: id_token.chatgpt_account_id.clone(),
            id_token,
            access_token: access_token.to_string(),
            refresh_token: String::new(),
        }),
        last_refresh: Some(Utc::now()),
        agent_identity: None,
    })
}

pub mod default_client {
    use codex_client::CodexHttpClient;

    pub use codex_client::CodexRequestBuilder;

    pub const DEFAULT_ORIGINATOR: &str = "codex_cli_rs";
    pub const CODEX_INTERNAL_ORIGINATOR_OVERRIDE_ENV_VAR: &str =
        "CODEX_INTERNAL_ORIGINATOR_OVERRIDE";
    pub const RESIDENCY_HEADER_NAME: &str = "x-openai-internal-codex-residency";

    #[derive(Clone, Debug)]
    pub struct Originator {
        pub value: String,
        pub header_value: http::HeaderValue,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum SetOriginatorError {
        #[error("invalid originator header value")]
        InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    }

    pub fn set_default_originator(_value: String) -> Result<(), SetOriginatorError> {
        Ok(())
    }

    pub fn set_default_client_residency_requirement(
        _enforce_residency: Option<codex_config::ResidencyRequirement>,
    ) {
    }

    pub fn originator() -> Originator {
        Originator {
            value: DEFAULT_ORIGINATOR.to_string(),
            header_value: http::HeaderValue::from_static(DEFAULT_ORIGINATOR),
        }
    }

    pub fn is_first_party_originator(originator_value: &str) -> bool {
        matches!(originator_value, "codex_cli_rs" | "codex_cli_rs_wasm")
    }

    pub fn is_first_party_chat_originator(originator_value: &str) -> bool {
        is_first_party_originator(originator_value)
    }

    pub fn get_codex_user_agent() -> String {
        format!("codex_cli_rs/{}", env!("CARGO_PKG_VERSION"))
    }

    pub fn create_client() -> CodexHttpClient {
        CodexHttpClient::new(build_reqwest_client())
    }

    pub fn build_reqwest_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    pub fn try_build_reqwest_client()
    -> Result<reqwest::Client, codex_client::BuildCustomCaTransportError> {
        Ok(build_reqwest_client())
    }

    pub fn default_headers() -> http::HeaderMap {
        let mut headers = http::HeaderMap::new();
        let _ = headers.insert(
            http::header::USER_AGENT,
            http::HeaderValue::from_str(&get_codex_user_agent())
                .unwrap_or_else(|_| http::HeaderValue::from_static("codex_cli_rs")),
        );
        headers
    }
}

pub mod auth {
    pub use super::AgentIdentityAuth;
    pub use super::AgentIdentityAuthRecord;
    pub use super::AuthConfig;
    pub use super::AuthDotJson;
    pub use super::AuthManager;
    pub use super::AuthManagerConfig;
    pub use super::CLIENT_ID;
    pub use super::CODEX_ACCESS_TOKEN_ENV_VAR;
    pub use super::CODEX_API_KEY_ENV_VAR;
    pub use super::CodexAuth;
    pub use super::ExternalAuth;
    pub use super::ExternalAuthChatgptMetadata;
    pub use super::ExternalAuthRefreshContext;
    pub use super::ExternalAuthRefreshReason;
    pub use super::ExternalAuthTokens;
    pub use super::OPENAI_API_KEY_ENV_VAR;
    pub use super::REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR;
    pub use super::REVOKE_TOKEN_URL_OVERRIDE_ENV_VAR;
    pub use super::RefreshTokenError;
    pub use super::UnauthorizedRecovery;
    pub use super::default_client;
    pub use super::enforce_login_restrictions;
    pub use super::load_auth_dot_json;
    pub use super::login_with_access_token;
    pub use super::login_with_api_key;
    pub use super::logout;
    pub use super::logout_with_revoke;
    pub use super::read_codex_access_token_from_env;
    pub use super::read_openai_api_key_from_env;
    pub use super::save_auth;

    pub mod error {
        pub use codex_protocol::auth::RefreshTokenFailedError;
        pub use codex_protocol::auth::RefreshTokenFailedReason;
    }
}
