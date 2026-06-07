#[derive(Clone, Debug, Default)]
pub struct LogDbLayer;

impl LogDbLayer {
    pub async fn flush(&self) {}
}
