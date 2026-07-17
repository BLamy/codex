use tokio::sync::mpsc::UnboundedSender;

use crate::app_event::AppEvent;

#[derive(Clone)]
pub(crate) struct AppEventSender {
    pub(crate) app_event_tx: UnboundedSender<AppEvent>,
}

impl AppEventSender {
    pub(crate) fn new(app_event_tx: UnboundedSender<AppEvent>) -> Self {
        Self { app_event_tx }
    }

    pub(crate) fn send(&self, event: AppEvent) {
        let _ = self.app_event_tx.send(event);
    }
}
