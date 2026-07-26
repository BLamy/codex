//! In-memory authentication storage for the browser build.
//!
//! Durable credentials are owned by the Almost Node host. This store only
//! preserves the current wasm process' auth snapshot so upstream AuthManager
//! semantics remain intact between requests.

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use super::BedrockApiKeyAuth;
use crate::token_data::TokenData;
use codex_config::types::AuthCredentialsStoreMode;
pub use codex_config::types::AuthKeyringBackendKind;
use codex_protocol::account::PlanType as AccountPlanType;
use codex_protocol::auth::AuthMode;

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
    pub agent_identity: Option<AgentIdentityStorage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personal_access_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bedrock_api_key: Option<BedrockApiKeyAuth>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum AgentIdentityStorage {
    Jwt(String),
    Record(AgentIdentityAuthRecord),
}

impl AgentIdentityStorage {
    pub fn has_auth_material(&self) -> bool {
        match self {
            Self::Jwt(jwt) => !jwt.trim().is_empty(),
            Self::Record(record) => {
                !record.agent_runtime_id.trim().is_empty()
                    && !record.agent_private_key.trim().is_empty()
            }
        }
    }

    pub(crate) fn as_record(&self) -> Option<&AgentIdentityAuthRecord> {
        match self {
            Self::Jwt(_) => None,
            Self::Record(record) => Some(record),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct AgentIdentityAuthRecord {
    pub agent_runtime_id: String,
    pub agent_private_key: String,
    pub account_id: String,
    pub chatgpt_user_id: String,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_non_empty_string",
        serialize_with = "serialize_optional_string_as_empty"
    )]
    pub email: Option<String>,
    pub plan_type: AccountPlanType,
    pub chatgpt_account_is_fedramp: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}

fn deserialize_optional_non_empty_string<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|value| value.filter(|value| !value.is_empty()))
}

fn serialize_optional_string_as_empty<S>(
    value: &Option<String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    value.as_deref().unwrap_or_default().serialize(serializer)
}

impl AgentIdentityAuthRecord {
    pub(crate) fn from_agent_identity_jwt(_jwt: &str) -> std::io::Result<Self> {
        Err(std::io::Error::other(
            "agent identity JWT verification is unavailable in the browser runtime",
        ))
    }
}

pub(super) fn get_auth_file(codex_home: &Path) -> PathBuf {
    codex_home.join("auth.json")
}

pub(super) fn delete_file_if_exists(codex_home: &Path) -> std::io::Result<bool> {
    Ok(auth_store()
        .lock()
        .map_err(|_| std::io::Error::other("failed to lock browser auth storage"))?
        .remove(codex_home)
        .is_some())
}

pub(super) trait AuthStorageBackend: Debug + Send + Sync {
    fn load(&self) -> std::io::Result<Option<AuthDotJson>>;
    fn save(&self, auth: &AuthDotJson) -> std::io::Result<()>;
    fn delete(&self) -> std::io::Result<bool>;
}

fn auth_store() -> &'static Mutex<HashMap<PathBuf, AuthDotJson>> {
    static STORE: LazyLock<Mutex<HashMap<PathBuf, AuthDotJson>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    &STORE
}

#[derive(Clone, Debug)]
struct BrowserAuthStorage {
    codex_home: PathBuf,
}

impl AuthStorageBackend for BrowserAuthStorage {
    fn load(&self) -> std::io::Result<Option<AuthDotJson>> {
        Ok(auth_store()
            .lock()
            .map_err(|_| std::io::Error::other("failed to lock browser auth storage"))?
            .get(&self.codex_home)
            .cloned())
    }

    fn save(&self, auth: &AuthDotJson) -> std::io::Result<()> {
        auth_store()
            .lock()
            .map_err(|_| std::io::Error::other("failed to lock browser auth storage"))?
            .insert(self.codex_home.clone(), auth.clone());
        Ok(())
    }

    fn delete(&self) -> std::io::Result<bool> {
        delete_file_if_exists(&self.codex_home)
    }
}

pub(super) fn create_auth_storage(
    codex_home: PathBuf,
    _mode: AuthCredentialsStoreMode,
    _keyring_backend_kind: AuthKeyringBackendKind,
) -> Arc<dyn AuthStorageBackend> {
    Arc::new(BrowserAuthStorage { codex_home })
}
