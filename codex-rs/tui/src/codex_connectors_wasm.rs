pub mod metadata {
    use codex_app_server_protocol::AppInfo;

    pub fn connector_display_label(connector: &AppInfo) -> String {
        connector.name.clone()
    }

    pub fn connector_mention_slug(connector: &AppInfo) -> String {
        connector_mention_slug_from_name(&connector_display_label(connector))
    }

    pub fn connector_mention_slug_from_name(name: &str) -> String {
        name.to_ascii_lowercase()
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}
