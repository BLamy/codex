use serde::Deserialize;
use serde::Serialize;

/// Registry key envelope retained in the browser-facing protocol surface.
///
/// Noise rendezvous transport is native-only, but these values remain
/// serializable so shared app-server protocol types compile unchanged.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoiseChannelPublicKey {
    suite: String,
    x25519_public_key: String,
    mlkem768_public_key: String,
}

#[derive(Clone, Debug)]
pub struct NoiseChannelIdentity;

impl NoiseChannelIdentity {
    pub fn generate() -> Result<Self, NoiseChannelError> {
        Err(NoiseChannelError::Unavailable)
    }

    pub fn public_key(&self) -> NoiseChannelPublicKey {
        unreachable!("Noise rendezvous is unavailable in browser wasm")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NoiseChannelError {
    #[error("Noise rendezvous is unavailable in browser wasm")]
    Unavailable,
}
