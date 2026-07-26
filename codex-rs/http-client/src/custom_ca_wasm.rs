use std::fmt;
use std::io;

/// Browser transports use the user agent's trust store, so custom CA bundles
/// cannot be installed by the wasm client.
#[derive(Debug)]
pub struct BuildCustomCaTransportError {
    source: reqwest::Error,
}

impl fmt::Display for BuildCustomCaTransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to build browser reqwest client; custom CA bundles are not supported on wasm32: {}",
            self.source
        )
    }
}

impl std::error::Error for BuildCustomCaTransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<BuildCustomCaTransportError> for io::Error {
    fn from(error: BuildCustomCaTransportError) -> Self {
        io::Error::other(error)
    }
}

pub fn build_reqwest_client_with_custom_ca(
    builder: reqwest::ClientBuilder,
) -> Result<reqwest::Client, BuildCustomCaTransportError> {
    builder
        .build()
        .map_err(|source| BuildCustomCaTransportError { source })
}
