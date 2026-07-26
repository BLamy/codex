use crate::history_cell::PlainHistoryCell;
use crate::legacy_core::config::Config;
use crate::session_state::SessionNetworkProxyRuntime;
use ratatui::style::Stylize;
use ratatui::text::Line;

pub(crate) fn new_debug_config_output(
    _config: &Config,
    _session_network_proxy: Option<&SessionNetworkProxyRuntime>,
) -> PlainHistoryCell {
    PlainHistoryCell::new(vec![
        "/debug-config".magenta().into(),
        Line::from(""),
        "Config diagnostics are running in the browser wasm host.".into(),
        "Host config layers are not available until the browser session provides them.".dim().into(),
    ])
}
