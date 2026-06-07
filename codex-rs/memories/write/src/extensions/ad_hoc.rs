use crate::memory_extensions_root;
use std::path::Path;

pub(super) const INSTRUCTIONS: &str =
    include_str!("../../templates/extensions/ad_hoc/instructions.md");

pub(super) async fn seed_instructions(memory_root: &Path) -> std::io::Result<()> {
    let extension_root = memory_extensions_root(memory_root).join("ad_hoc");
    let instructions_path = extension_root.join("instructions.md");

    crate::fs::create_dir_all(&extension_root).await?;
    match crate::fs::write_new(&instructions_path, INSTRUCTIONS.as_bytes()).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
#[path = "ad_hoc_tests.rs"]
mod tests;
