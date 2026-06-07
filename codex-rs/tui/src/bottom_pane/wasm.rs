use std::path::PathBuf;
use std::time::Duration;

mod chat_composer;
mod chat_composer_history;
mod command_popup;
mod file_search_popup;
mod footer;
mod mentions_v2;
pub(crate) mod paste_burst;
pub(crate) mod popup_consts;
pub(crate) mod prompt_args;
mod scroll_state;
mod selection_popup_common;
mod skill_popup;
pub(crate) mod slash_commands;
pub(crate) mod textarea;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LocalImageAttachment {
    pub(crate) placeholder: String,
    pub(crate) path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MentionBinding {
    pub(crate) sigil: char,
    pub(crate) mention: String,
    pub(crate) path: String,
}

pub(crate) const QUIT_SHORTCUT_TIMEOUT: Duration = Duration::from_secs(1);

pub(crate) use chat_composer::ChatComposer;
pub(crate) use chat_composer::InputResult;
pub(crate) use chat_composer::QueuedInputAction;
