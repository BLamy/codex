#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LinkedMention {
    pub(crate) sigil: char,
    pub(crate) mention: String,
    pub(crate) path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecodedHistoryText {
    pub(crate) text: String,
    pub(crate) mentions: Vec<LinkedMention>,
}

pub(crate) fn decode_history_mentions_with_at_mentions(
    text: &str,
    _at_mentions_enabled: bool,
) -> DecodedHistoryText {
    DecodedHistoryText {
        text: text.to_string(),
        mentions: Vec::new(),
    }
}
