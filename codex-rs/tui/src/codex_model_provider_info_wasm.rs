use codex_protocol::config_types::ModelProviderAuthInfo;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

pub const OPENAI_PROVIDER_ID: &str = "openai";
pub const AMAZON_BEDROCK_PROVIDER_ID: &str = "amazon-bedrock";
pub const CHATGPT_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WireApi {
    #[default]
    Responses,
}

impl fmt::Display for WireApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Responses => f.write_str("responses"),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModelProviderAwsAuthInfo {
    pub profile: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModelProviderInfo {
    pub name: String,
    pub base_url: Option<String>,
    pub env_key: Option<String>,
    pub env_key_instructions: Option<String>,
    pub experimental_bearer_token: Option<String>,
    pub auth: Option<ModelProviderAuthInfo>,
    pub aws: Option<ModelProviderAwsAuthInfo>,
    pub wire_api: WireApi,
    pub query_params: Option<HashMap<String, String>>,
    pub http_headers: Option<HashMap<String, String>>,
    pub env_http_headers: Option<HashMap<String, String>>,
    pub request_max_retries: Option<u64>,
    pub stream_max_retries: Option<u64>,
    pub stream_idle_timeout_ms: Option<u64>,
    pub websocket_connect_timeout_ms: Option<u64>,
    pub requires_openai_auth: bool,
    pub supports_websockets: bool,
}

impl Default for ModelProviderInfo {
    fn default() -> Self {
        Self {
            name: "OpenAI".to_string(),
            base_url: Some("https://api.openai.com/v1".to_string()),
            env_key: Some("OPENAI_API_KEY".to_string()),
            env_key_instructions: None,
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Responses,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            websocket_connect_timeout_ms: None,
            requires_openai_auth: true,
            supports_websockets: false,
        }
    }
}

impl ModelProviderInfo {
    pub fn is_openai(&self) -> bool {
        self.name.eq_ignore_ascii_case("openai") || self.requires_openai_auth
    }

    pub fn create_openai_provider(openai_base_url: Option<String>) -> Self {
        Self {
            base_url: Some(openai_base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string())),
            ..Self::default()
        }
    }

    pub fn create_amazon_bedrock_provider(aws: Option<ModelProviderAwsAuthInfo>) -> Self {
        Self {
            name: "Amazon Bedrock".to_string(),
            base_url: Some("https://bedrock-mantle.us-east-1.api.aws/openai/v1".to_string()),
            env_key: None,
            aws,
            requires_openai_auth: false,
            ..Self::default()
        }
    }
}

pub fn built_in_model_providers(
    openai_base_url: Option<String>,
) -> HashMap<String, ModelProviderInfo> {
    [
        (
            OPENAI_PROVIDER_ID.to_string(),
            ModelProviderInfo::create_openai_provider(openai_base_url),
        ),
        (
            AMAZON_BEDROCK_PROVIDER_ID.to_string(),
            ModelProviderInfo::create_amazon_bedrock_provider(None),
        ),
    ]
    .into_iter()
    .collect()
}
