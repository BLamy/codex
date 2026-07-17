use crate::error::StreamError;
use crate::transport::ByteStream;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::timeout;

/// Minimal SSE helper that forwards raw `data:` frames as UTF-8 strings.
///
/// Errors and idle timeouts are sent as `Err(StreamError)` before the task exits.
pub fn sse_stream(
    stream: ByteStream,
    idle_timeout: Duration,
    tx: mpsc::Sender<Result<String, StreamError>>,
) {
    let task = async move {
        let mut stream = stream
            .map(|res| res.map_err(|e| StreamError::Stream(e.to_string())))
            .eventsource();

        loop {
            #[cfg(target_arch = "wasm32")]
            let _ = idle_timeout;

            #[cfg(not(target_arch = "wasm32"))]
            match timeout(idle_timeout, stream.next()).await {
                Ok(Some(Ok(ev))) => {
                    if tx.send(Ok(ev.data.clone())).await.is_err() {
                        return;
                    }
                }
                Ok(Some(Err(e))) => {
                    let _ = tx.send(Err(StreamError::Stream(e.to_string()))).await;
                    return;
                }
                Ok(None) => {
                    let _ = tx
                        .send(Err(StreamError::Stream(
                            "stream closed before completion".into(),
                        )))
                        .await;
                    return;
                }
                Err(_) => {
                    let _ = tx.send(Err(StreamError::Timeout)).await;
                    return;
                }
            }

            #[cfg(target_arch = "wasm32")]
            match stream.next().await {
                Some(Ok(ev)) => {
                    if tx.send(Ok(ev.data.clone())).await.is_err() {
                        return;
                    }
                }
                Some(Err(e)) => {
                    let _ = tx.send(Err(StreamError::Stream(e.to_string()))).await;
                    return;
                }
                None => {
                    let _ = tx
                        .send(Err(StreamError::Stream(
                            "stream closed before completion".into(),
                        )))
                        .await;
                    return;
                }
            }
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(task);

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(task);
}
