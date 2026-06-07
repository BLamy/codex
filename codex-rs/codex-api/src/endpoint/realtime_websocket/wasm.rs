use crate::endpoint::realtime_websocket::protocol::RealtimeAudioFrame;
use crate::endpoint::realtime_websocket::protocol::RealtimeEvent;
use crate::endpoint::realtime_websocket::protocol::RealtimeOutputModality;
use crate::endpoint::realtime_websocket::protocol::RealtimeSessionConfig;
use crate::endpoint::realtime_websocket::protocol::RealtimeSessionMode;
use crate::endpoint::realtime_websocket::protocol::RealtimeVoice;
use crate::error::ApiError;
use crate::provider::Provider;
use http::HeaderMap;

#[derive(Clone, Debug)]
pub struct RealtimeWebsocketConnection {
    writer: RealtimeWebsocketWriter,
    events: RealtimeWebsocketEvents,
}

#[derive(Clone, Debug)]
pub struct RealtimeWebsocketWriter;

#[derive(Clone, Debug)]
pub struct RealtimeWebsocketEvents;

impl RealtimeWebsocketConnection {
    pub async fn send_audio_frame(&self, frame: RealtimeAudioFrame) -> Result<(), ApiError> {
        self.writer.send_audio_frame(frame).await
    }

    pub async fn send_conversation_item_create(&self, text: String) -> Result<(), ApiError> {
        self.writer.send_conversation_item_create(text).await
    }

    pub async fn send_conversation_function_call_output(
        &self,
        call_id: String,
        output_text: String,
    ) -> Result<(), ApiError> {
        self.writer
            .send_conversation_function_call_output(call_id, output_text)
            .await
    }

    pub async fn close(&self) -> Result<(), ApiError> {
        self.writer.close().await
    }

    pub async fn next_event(&self) -> Result<Option<RealtimeEvent>, ApiError> {
        self.events.next_event().await
    }

    pub fn writer(&self) -> RealtimeWebsocketWriter {
        self.writer.clone()
    }

    pub fn events(&self) -> RealtimeWebsocketEvents {
        self.events.clone()
    }
}

impl RealtimeWebsocketWriter {
    pub async fn send_audio_frame(&self, _frame: RealtimeAudioFrame) -> Result<(), ApiError> {
        Err(unsupported())
    }

    pub async fn send_conversation_item_create(&self, _text: String) -> Result<(), ApiError> {
        Err(unsupported())
    }

    pub async fn send_conversation_function_call_output(
        &self,
        _call_id: String,
        _output_text: String,
    ) -> Result<(), ApiError> {
        Err(unsupported())
    }

    pub async fn send_response_create(&self) -> Result<(), ApiError> {
        Err(unsupported())
    }

    pub async fn send_session_update(
        &self,
        _instructions: String,
        _session_mode: RealtimeSessionMode,
        _output_modality: RealtimeOutputModality,
        _voice: RealtimeVoice,
    ) -> Result<(), ApiError> {
        Err(unsupported())
    }

    pub async fn close(&self) -> Result<(), ApiError> {
        Ok(())
    }

    pub async fn send_payload(&self, _payload: String) -> Result<(), ApiError> {
        Err(unsupported())
    }
}

impl RealtimeWebsocketEvents {
    pub async fn next_event(&self) -> Result<Option<RealtimeEvent>, ApiError> {
        Err(unsupported())
    }
}

pub struct RealtimeWebsocketClient {
    _provider: Provider,
}

impl RealtimeWebsocketClient {
    pub fn new(provider: Provider) -> Self {
        Self {
            _provider: provider,
        }
    }

    pub async fn connect(
        &self,
        _config: RealtimeSessionConfig,
        _extra_headers: HeaderMap,
        _default_headers: HeaderMap,
    ) -> Result<RealtimeWebsocketConnection, ApiError> {
        Err(unsupported())
    }

    pub async fn connect_webrtc_sideband(
        &self,
        _config: RealtimeSessionConfig,
        _call_id: &str,
        _extra_headers: HeaderMap,
        _default_headers: HeaderMap,
    ) -> Result<RealtimeWebsocketConnection, ApiError> {
        Err(unsupported())
    }
}

fn unsupported() -> ApiError {
    ApiError::Stream(
        "realtime websocket transport is not available on wasm32 without a browser WebSocket host shim"
            .to_string(),
    )
}
