use std::future::Future;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn spawn_background<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn spawn_background<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}
