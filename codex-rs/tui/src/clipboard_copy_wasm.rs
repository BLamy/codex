pub(crate) struct ClipboardLease;

impl ClipboardLease {
    #[cfg(test)]
    pub(crate) fn test() -> Self {
        Self
    }
}

pub(crate) fn copy_to_clipboard(_text: &str) -> Result<Option<ClipboardLease>, String> {
    Ok(None)
}
