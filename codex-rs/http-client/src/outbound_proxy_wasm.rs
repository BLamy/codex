//! Browser-safe outbound proxy facade.
//!
//! Browser networking delegates route and proxy selection to the user agent.
//! This module preserves the native factory API while leaving the reqwest
//! builder untouched.

use std::fmt;
use std::io;

use thiserror::Error;

use crate::custom_ca_wasm::BuildCustomCaTransportError;
use crate::custom_ca_wasm::build_reqwest_client_with_custom_ca;
use crate::default_client::HttpClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientRouteClass {
    Auth,
    Api,
    WebSocket,
    Other,
}

impl fmt::Display for ClientRouteClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Auth => "auth",
            Self::Api => "api",
            Self::WebSocket => "wss",
            Self::Other => "other",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteFailureClass {
    ProxyResolutionUnavailable,
    ConnectTimeout,
    ProxyAuthenticationRequired,
    TlsError,
    InvalidProxyConfig,
    UnsupportedProxyScheme,
    ResolverError,
}

impl fmt::Display for RouteFailureClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ProxyResolutionUnavailable => "proxy_resolution_unavailable",
            Self::ConnectTimeout => "connect_timeout",
            Self::ProxyAuthenticationRequired => "proxy_407",
            Self::TlsError => "tls_error",
            Self::InvalidProxyConfig => "invalid_proxy_config",
            Self::UnsupportedProxyScheme => "unsupported_proxy_scheme",
            Self::ResolverError => "resolver_error",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboundProxyPolicy {
    ReqwestDefault,
    RespectSystemProxy,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum OutboundProxyRoute {
    TransportDefault,
    Direct,
    Proxy {
        url: String,
        no_proxy: Option<String>,
    },
}

impl fmt::Debug for OutboundProxyRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TransportDefault => f.write_str("TransportDefault"),
            Self::Direct => f.write_str("Direct"),
            Self::Proxy { .. } => f
                .debug_struct("Proxy")
                .field("url", &"<redacted>")
                .field("no_proxy", &"<redacted>")
                .finish(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpClientFactory {
    outbound_proxy_policy: OutboundProxyPolicy,
}

impl HttpClientFactory {
    pub const fn new(outbound_proxy_policy: OutboundProxyPolicy) -> Self {
        Self {
            outbound_proxy_policy,
        }
    }

    pub const fn outbound_proxy_policy(&self) -> OutboundProxyPolicy {
        self.outbound_proxy_policy
    }

    pub fn resolve_proxy_route(&self, _request_url: &str) -> OutboundProxyRoute {
        OutboundProxyRoute::TransportDefault
    }

    pub async fn resolve_proxy_route_async(
        &self,
        request_url: String,
    ) -> io::Result<OutboundProxyRoute> {
        Ok(self.resolve_proxy_route(&request_url))
    }

    pub fn build_client(
        &self,
        request_url: &str,
        route_class: ClientRouteClass,
    ) -> Result<HttpClient, BuildRouteAwareHttpClientError> {
        self.build_reqwest_client(reqwest::Client::builder(), request_url, route_class)
            .map(HttpClient::new)
    }

    pub fn build_client_without_request_logging(
        &self,
        request_url: &str,
        route_class: ClientRouteClass,
    ) -> Result<HttpClient, BuildRouteAwareHttpClientError> {
        self.build_reqwest_client(reqwest::Client::builder(), request_url, route_class)
            .map(HttpClient::new_without_request_logging)
    }

    pub fn build_reqwest_client(
        &self,
        builder: reqwest::ClientBuilder,
        _request_url: &str,
        _route_class: ClientRouteClass,
    ) -> Result<reqwest::Client, BuildRouteAwareHttpClientError> {
        build_reqwest_client_with_custom_ca(builder).map_err(Into::into)
    }
}

#[derive(Debug, Error)]
pub enum BuildRouteAwareHttpClientError {
    #[error(transparent)]
    CustomCa(#[from] BuildCustomCaTransportError),

    #[error("Failed to configure outbound proxy selected for {route_class}")]
    InvalidProxyConfig { route_class: ClientRouteClass },
}

impl From<BuildRouteAwareHttpClientError> for io::Error {
    fn from(error: BuildRouteAwareHttpClientError) -> Self {
        match error {
            BuildRouteAwareHttpClientError::CustomCa(error) => error.into(),
            BuildRouteAwareHttpClientError::InvalidProxyConfig { .. } => io::Error::other(error),
        }
    }
}
