use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppBranding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub logo_url_dark: Option<String>,
    pub distribution_channel: Option<String>,
    pub branding: Option<AppBranding>,
    pub app_metadata: Option<AppMetadata>,
    pub labels: Option<HashMap<String, String>>,
    pub install_url: Option<String>,
    pub is_accessible: bool,
    pub is_enabled: bool,
    pub plugin_display_names: Vec<String>,
}
