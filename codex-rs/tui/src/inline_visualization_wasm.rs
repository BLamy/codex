//! Browser-safe fallback for assistant-authored inline visualizations.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

pub(crate) const DIRECTIVE_PREFIX: &str = "::codex-inline-vis{";

#[derive(Clone, Debug, Default)]
pub(crate) struct InlineVisualizationContext;

impl InlineVisualizationContext {
    pub(crate) fn new(
        _codex_home: &Path,
        _thread_id: codex_protocol::ThreadId,
    ) -> Option<Self> {
        None
    }

    pub(crate) fn from_config(
        _config: &crate::legacy_core::config::Config,
        _thread_id: codex_protocol::ThreadId,
    ) -> Option<Self> {
        None
    }
}

pub(crate) struct TrustedFileLink {
    pub(crate) destination: url::Url,
    pub(crate) markdown_label: String,
    pub(crate) display_label: String,
    pub(crate) markdown_destination_label: String,
}

pub(crate) struct InlineVisualizationRewrite<'a> {
    pub(crate) markdown: Cow<'a, str>,
    pub(crate) trusted_file_links: HashMap<String, TrustedFileLink>,
}

pub(crate) fn rewrite_inline_visualizations<'a>(
    markdown: &'a str,
    _context: Option<&InlineVisualizationContext>,
) -> InlineVisualizationRewrite<'a> {
    if !markdown.contains(DIRECTIVE_PREFIX) {
        return InlineVisualizationRewrite {
            markdown: Cow::Borrowed(markdown),
            trusted_file_links: HashMap::new(),
        };
    }

    let rewritten = markdown
        .split_inclusive('\n')
        .map(|source_line| {
            let (line, newline) = source_line
                .strip_suffix('\n')
                .map_or((source_line, ""), |line| (line, "\n"));
            if line.trim().starts_with(DIRECTIVE_PREFIX) && line.trim().ends_with('}') {
                format!("_Visualization unavailable in the browser sandbox._{newline}")
            } else {
                source_line.to_string()
            }
        })
        .collect::<String>();

    InlineVisualizationRewrite {
        markdown: Cow::Owned(rewritten),
        trusted_file_links: HashMap::new(),
    }
}
