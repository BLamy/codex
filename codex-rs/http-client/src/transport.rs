#[cfg(not(target_arch = "wasm32"))]
use crate::default_client::HttpClient;
#[cfg(not(target_arch = "wasm32"))]
use crate::default_client::RequestBuilder;
use crate::error::TransportError;
use crate::request::Request;
use crate::request::RequestBody;
use crate::request::Response;
#[cfg(target_arch = "wasm32")]
use base64::Engine;
#[cfg(target_arch = "wasm32")]
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use bytes::Bytes;
use futures::StreamExt;
use futures::stream::BoxStream;
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
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::AtomicBool;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::Ordering;
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

pub type ByteStream = BoxStream<'static, Result<Bytes, TransportError>>;

pub struct StreamResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub bytes: ByteStream,
}

pub trait HttpTransport: Send + Sync {
    fn execute(
        &self,
        req: Request,
    ) -> impl std::future::Future<Output = Result<Response, TransportError>> + Send;
    fn stream(
        &self,
        req: Request,
    ) -> impl std::future::Future<Output = Result<StreamResponse, TransportError>> + Send;
}

#[derive(Clone, Debug)]
pub struct ReqwestTransport {
    #[cfg(not(target_arch = "wasm32"))]
    client: HttpClient,
}

impl ReqwestTransport {
    pub fn new(client: reqwest::Client) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            return Self {
                client: HttpClient::new(client),
            };
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = client;
            Self {}
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn build(&self, req: Request) -> Result<RequestBuilder, TransportError> {
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
}

#[cfg(target_arch = "wasm32")]
fn prepare_host_fetch_request(req: Request) -> Result<HostNetworkFetchRequest, TransportError> {
    let prepared = req.prepare_body_for_send().map_err(TransportError::Build)?;
    Ok(HostNetworkFetchRequest {
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
    })
}

fn request_body_for_trace(req: &Request) -> String {
    match req.body.as_ref() {
        Some(RequestBody::Json(body)) => body.to_string(),
        Some(RequestBody::EncodedJson(body)) => {
            String::from_utf8_lossy(body.trace_bytes()).into_owned()
        }
        Some(RequestBody::Raw(body)) => format!("<raw body: {} bytes>", body.len()),
        None => String::new(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpTransport for ReqwestTransport {
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
impl HttpTransport for ReqwestTransport {
    fn execute(
        &self,
        req: Request,
    ) -> impl std::future::Future<Output = Result<Response, TransportError>> + Send {
        if enabled!(Level::TRACE) {
            trace!(
                "{} to {}: {}",
                req.method,
                req.url,
                request_body_for_trace(&req)
            );
        }

        bridge_local(async move { execute_with_host(req).await })
    }

    fn stream(
        &self,
        req: Request,
    ) -> impl std::future::Future<Output = Result<StreamResponse, TransportError>> + Send {
        if enabled!(Level::TRACE) {
            trace!(
                "{} to {}: {}",
                req.method,
                req.url,
                request_body_for_trace(&req)
            );
        }

        async move {
            let fetch_response =
                bridge_local(async move { open_stream_with_host(req).await }).await?;
            let status = match StatusCode::from_u16(fetch_response.status) {
                Ok(status) => status,
                Err(error) => {
                    cancel_host_fetch_stream_bridged(fetch_response.stream_id).await;
                    return Err(TransportError::Network(error.to_string()));
                }
            };
            let headers = map_to_headers(fetch_response.headers);
            if !status.is_success() {
                cancel_host_fetch_stream_bridged(fetch_response.stream_id).await;
                return Err(TransportError::Http {
                    status,
                    url: Some(fetch_response.url),
                    headers: Some(headers),
                    body: None,
                });
            }

            Ok(StreamResponse {
                status,
                headers,
                bytes: host_fetch_byte_stream(fetch_response.stream_id),
            })
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn execute_with_host(req: Request) -> Result<Response, TransportError> {
    let params = prepare_host_fetch_request(req)?;
    let fetch_response: HostNetworkFetchResponse =
        host_request_json("network/fetch", &params).await?;
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

#[cfg(target_arch = "wasm32")]
async fn open_stream_with_host(
    req: Request,
) -> Result<HostNetworkFetchStreamOpenResponse, TransportError> {
    let params = prepare_host_fetch_request(req)?;
    host_request_json("network/fetchStreamOpen", &params).await
}

#[cfg(target_arch = "wasm32")]
fn bridge_local<T>(
    future: impl std::future::Future<Output = Result<T, TransportError>> + 'static,
) -> impl std::future::Future<Output = Result<T, TransportError>> + Send
where
    T: Send + 'static,
{
    let (tx, rx) = tokio::sync::oneshot::channel();
    wasm_bindgen_futures::spawn_local(async move {
        let _ = tx.send(future.await);
    });
    async move {
        rx.await.map_err(|_| {
            TransportError::Network("browser host request task was cancelled".to_string())
        })?
    }
}

#[cfg(target_arch = "wasm32")]
fn host_fetch_byte_stream(stream_id: String) -> ByteStream {
    let complete = Arc::new(AtomicBool::new(false));
    let producer_complete = Arc::clone(&complete);
    let producer_stream_id = stream_id.clone();
    let (tx, rx) = tokio::sync::mpsc::channel(8);

    wasm_bindgen_futures::spawn_local(async move {
        loop {
            let response: Result<HostNetworkFetchStreamReadResponse, TransportError> =
                host_request_json(
                    "network/fetchStreamRead",
                    &HostNetworkFetchStreamRequest {
                        stream_id: &producer_stream_id,
                    },
                )
                .await;
            let response = match response {
                Ok(response) => response,
                Err(error) => {
                    let _ = tx.send(Err(error)).await;
                    cancel_host_fetch_stream(&producer_stream_id).await;
                    producer_complete.store(true, Ordering::Release);
                    return;
                }
            };
            if response.done {
                producer_complete.store(true, Ordering::Release);
                return;
            }
            if response.chunk_base64.is_empty() {
                let _ = tx
                    .send(Err(TransportError::Network(
                        "network/fetchStreamRead returned an empty non-final chunk".to_string(),
                    )))
                    .await;
                cancel_host_fetch_stream(&producer_stream_id).await;
                producer_complete.store(true, Ordering::Release);
                return;
            }
            let chunk = decode_body(&response.chunk_base64);
            let is_error = chunk.is_err();
            if tx.send(chunk).await.is_err() {
                return;
            }
            if is_error {
                cancel_host_fetch_stream(&producer_stream_id).await;
                producer_complete.store(true, Ordering::Release);
                return;
            }
        }
    });

    let state = HostFetchReceiverState {
        rx,
        _cancel: HostFetchCancelGuard {
            stream_id,
            complete,
        },
    };
    futures::stream::unfold(state, |mut state| async move {
        state.rx.recv().await.map(|item| (item, state))
    })
    .boxed()
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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostNetworkFetchStreamOpenResponse {
    stream_id: String,
    url: String,
    status: u16,
    #[serde(default)]
    #[serde(rename = "statusText")]
    _status_text: String,
    #[serde(default)]
    headers: HashMap<String, String>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostNetworkFetchStreamRequest<'a> {
    stream_id: &'a str,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostNetworkFetchStreamReadResponse {
    chunk_base64: String,
    done: bool,
}

#[cfg(target_arch = "wasm32")]
struct HostFetchReceiverState {
    rx: tokio::sync::mpsc::Receiver<Result<Bytes, TransportError>>,
    _cancel: HostFetchCancelGuard,
}

#[cfg(target_arch = "wasm32")]
struct HostFetchCancelGuard {
    stream_id: String,
    complete: Arc<AtomicBool>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for HostFetchCancelGuard {
    fn drop(&mut self) {
        if self.complete.load(Ordering::Acquire) {
            return;
        }
        let stream_id = self.stream_id.clone();
        wasm_bindgen_futures::spawn_local(async move {
            cancel_host_fetch_stream(&stream_id).await;
        });
    }
}

#[cfg(target_arch = "wasm32")]
async fn cancel_host_fetch_stream(stream_id: &str) {
    let _: Result<serde_json::Value, TransportError> = host_request_json(
        "network/fetchStreamCancel",
        &HostNetworkFetchStreamRequest { stream_id },
    )
    .await;
}

#[cfg(target_arch = "wasm32")]
async fn cancel_host_fetch_stream_bridged(stream_id: String) {
    let _ = bridge_local(async move {
        cancel_host_fetch_stream(&stream_id).await;
        Ok(())
    })
    .await;
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
