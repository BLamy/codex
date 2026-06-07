use crate::default_client::CodexHttpClient;
#[cfg(not(target_arch = "wasm32"))]
use crate::default_client::CodexRequestBuilder;
use crate::error::TransportError;
use crate::request::Request;
use crate::request::RequestBody;
use crate::request::Response;
use async_trait::async_trait;
#[cfg(target_arch = "wasm32")]
use base64::Engine;
#[cfg(target_arch = "wasm32")]
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use bytes::Bytes;
#[cfg(not(target_arch = "wasm32"))]
use futures::StreamExt;
#[cfg(target_arch = "wasm32")]
use futures::stream;
#[cfg(not(target_arch = "wasm32"))]
use futures::stream::BoxStream;
#[cfg(target_arch = "wasm32")]
use futures::stream::LocalBoxStream;
use http::HeaderMap;
#[cfg(target_arch = "wasm32")]
use http::HeaderName;
#[cfg(target_arch = "wasm32")]
use http::HeaderValue;
#[cfg(not(target_arch = "wasm32"))]
use http::Method;
use http::StatusCode;
#[cfg(target_arch = "wasm32")]
use js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use js_sys::Reflect;
#[cfg(target_arch = "wasm32")]
use serde::Deserialize;
#[cfg(target_arch = "wasm32")]
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use serde::de::DeserializeOwned;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
use tracing::Level;
use tracing::enabled;
use tracing::trace;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis, js_name = __almostnodeCodexHostRequest, catch)]
    fn almostnode_codex_host_request(op: &str, params: JsValue) -> Result<Promise, JsValue>;
}

#[cfg(not(target_arch = "wasm32"))]
pub type ByteStream = BoxStream<'static, Result<Bytes, TransportError>>;
#[cfg(target_arch = "wasm32")]
pub type ByteStream = LocalBoxStream<'static, Result<Bytes, TransportError>>;

pub struct StreamResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub bytes: ByteStream,
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait HttpTransport: Send + Sync {
    async fn execute(&self, req: Request) -> Result<Response, TransportError>;
    async fn stream(&self, req: Request) -> Result<StreamResponse, TransportError>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
pub trait HttpTransport {
    async fn execute(&self, req: Request) -> Result<Response, TransportError>;
    async fn stream(&self, req: Request) -> Result<StreamResponse, TransportError>;
}

#[derive(Clone, Debug)]
pub struct ReqwestTransport {
    client: CodexHttpClient,
}

impl ReqwestTransport {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            client: CodexHttpClient::new(client),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn build(&self, req: Request) -> Result<CodexRequestBuilder, TransportError> {
        let prepared = req.prepare_body_for_send().map_err(TransportError::Build)?;

        let Request {
            method,
            url,
            headers: _,
            body: _,
            compression: _,
            timeout,
        } = req;

        let mut builder = self.client.request(
            Method::from_bytes(method.as_str().as_bytes()).unwrap_or(Method::GET),
            &url,
        );

        if let Some(timeout) = timeout {
            builder = builder.timeout(timeout);
        }

        builder = builder.headers(prepared.headers);
        if let Some(body) = prepared.body {
            builder = builder.body(body);
        }
        Ok(builder)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn map_error(err: reqwest::Error) -> TransportError {
        if err.is_timeout() {
            TransportError::Timeout
        } else {
            TransportError::Network(err.to_string())
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn fetch_with_host(
        &self,
        req: Request,
    ) -> Result<HostNetworkFetchResponse, TransportError> {
        let _client = &self.client;
        let prepared = req.prepare_body_for_send().map_err(TransportError::Build)?;
        let params = HostNetworkFetchRequest {
            url: req.url,
            method: req.method.to_string(),
            headers: headers_to_map(&prepared.headers),
            body_base64: prepared
                .body
                .as_ref()
                .map(|body| BASE64_STANDARD.encode(body.as_ref())),
            redirect: "follow",
            credentials: "include",
            retry_on_tailscale_recovery: true,
        };

        host_request_json("network/fetch", &params).await
    }
}

fn request_body_for_trace(req: &Request) -> String {
    match req.body.as_ref() {
        Some(RequestBody::Json(body)) => body.to_string(),
        Some(RequestBody::Raw(body)) => format!("<raw body: {} bytes>", body.len()),
        None => String::new(),
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl HttpTransport for ReqwestTransport {
    #[cfg(target_arch = "wasm32")]
    async fn execute(&self, req: Request) -> Result<Response, TransportError> {
        if enabled!(Level::TRACE) {
            trace!(
                "{} to {}: {}",
                req.method,
                req.url,
                request_body_for_trace(&req)
            );
        }

        let fetch_response = self.fetch_with_host(req).await?;
        let status = StatusCode::from_u16(fetch_response.status)
            .map_err(|err| TransportError::Network(err.to_string()))?;
        let headers = map_to_headers(fetch_response.headers);
        let bytes = decode_body(&fetch_response.body_base64)?;
        if !status.is_success() {
            let body = String::from_utf8(bytes.to_vec()).ok();
            return Err(TransportError::Http {
                status,
                url: Some(fetch_response.url),
                headers: Some(headers),
                body,
            });
        }
        Ok(Response {
            status,
            headers,
            body: bytes,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn execute(&self, req: Request) -> Result<Response, TransportError> {
        if enabled!(Level::TRACE) {
            trace!(
                "{} to {}: {}",
                req.method,
                req.url,
                request_body_for_trace(&req)
            );
        }

        let url = req.url.clone();
        let builder = self.build(req)?;
        let resp = builder.send().await.map_err(Self::map_error)?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = resp.bytes().await.map_err(Self::map_error)?;
        if !status.is_success() {
            let body = String::from_utf8(bytes.to_vec()).ok();
            return Err(TransportError::Http {
                status,
                url: Some(url),
                headers: Some(headers),
                body,
            });
        }
        Ok(Response {
            status,
            headers,
            body: bytes,
        })
    }

    #[cfg(target_arch = "wasm32")]
    async fn stream(&self, req: Request) -> Result<StreamResponse, TransportError> {
        if enabled!(Level::TRACE) {
            trace!(
                "{} to {}: {}",
                req.method,
                req.url,
                request_body_for_trace(&req)
            );
        }

        let fetch_response = self.fetch_with_host(req).await?;
        let status = StatusCode::from_u16(fetch_response.status)
            .map_err(|err| TransportError::Network(err.to_string()))?;
        let headers = map_to_headers(fetch_response.headers);
        let bytes = decode_body(&fetch_response.body_base64)?;
        if !status.is_success() {
            let body = String::from_utf8(bytes.to_vec()).ok();
            return Err(TransportError::Http {
                status,
                url: Some(fetch_response.url),
                headers: Some(headers),
                body,
            });
        }

        Ok(StreamResponse {
            status,
            headers,
            bytes: Box::pin(stream::once(async move { Ok(bytes) })),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn stream(&self, req: Request) -> Result<StreamResponse, TransportError> {
        if enabled!(Level::TRACE) {
            trace!(
                "{} to {}: {}",
                req.method,
                req.url,
                request_body_for_trace(&req)
            );
        }

        let url = req.url.clone();
        let builder = self.build(req)?;
        let resp = builder.send().await.map_err(Self::map_error)?;
        let status = resp.status();
        let headers = resp.headers().clone();
        if !status.is_success() {
            let body = resp.text().await.ok();
            return Err(TransportError::Http {
                status,
                url: Some(url),
                headers: Some(headers),
                body,
            });
        }
        let stream = resp
            .bytes_stream()
            .map(|result| result.map_err(Self::map_error));
        Ok(StreamResponse {
            status,
            headers,
            bytes: Box::pin(stream),
        })
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostNetworkFetchRequest {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body_base64: Option<String>,
    redirect: &'static str,
    credentials: &'static str,
    retry_on_tailscale_recovery: bool,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostNetworkFetchResponse {
    url: String,
    status: u16,
    #[serde(default)]
    #[serde(rename = "statusText")]
    _status_text: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    body_base64: String,
}

#[cfg(target_arch = "wasm32")]
async fn host_request_json<T, P>(op: &str, params: &P) -> Result<T, TransportError>
where
    T: DeserializeOwned,
    P: Serialize,
{
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    let js_params = params
        .serialize(&serializer)
        .map_err(|err| TransportError::Build(err.to_string()))?;
    let promise = almostnode_codex_host_request(op, js_params).map_err(js_error_to_transport)?;
    let value = JsFuture::from(promise)
        .await
        .map_err(js_error_to_transport)?;
    serde_wasm_bindgen::from_value(value).map_err(|err| TransportError::Network(err.to_string()))
}

#[cfg(target_arch = "wasm32")]
fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_string(), value.to_string()))
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn map_to_headers(headers: HashMap<String, String>) -> HeaderMap {
    let mut out = HeaderMap::new();
    for (name, value) in headers {
        let Ok(name) = HeaderName::from_bytes(name.as_bytes()) else {
            continue;
        };
        let Ok(value) = HeaderValue::from_str(&value) else {
            continue;
        };
        out.append(name, value);
    }
    out
}

#[cfg(target_arch = "wasm32")]
fn decode_body(body_base64: &str) -> Result<Bytes, TransportError> {
    BASE64_STANDARD
        .decode(body_base64)
        .map(Bytes::from)
        .map_err(|err| TransportError::Network(err.to_string()))
}

#[cfg(target_arch = "wasm32")]
fn js_error_to_transport(error: JsValue) -> TransportError {
    let message = js_error_property(&error, "message").unwrap_or_else(|| format!("{error:?}"));
    match js_error_property(&error, "code") {
        Some(code) if !code.is_empty() => TransportError::Network(format!("{code}: {message}")),
        _ => TransportError::Network(message),
    }
}

#[cfg(target_arch = "wasm32")]
fn js_error_property(error: &JsValue, name: &str) -> Option<String> {
    Reflect::get(error, &JsValue::from_str(name))
        .ok()
        .and_then(|value| value.as_string())
}
