use std::io;
use std::path::Path;
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_dir(path: impl AsRef<Path>) -> io::Result<()> {
    tokio::fs::create_dir(path).await
}

#[cfg(target_arch = "wasm32")]
pub async fn create_dir(path: impl AsRef<Path>) -> io::Result<()> {
    std::fs::create_dir(path)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    tokio::fs::read_to_string(path).await
}

#[cfg(target_arch = "wasm32")]
pub async fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    std::fs::read_to_string(path)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn symlink_metadata(path: impl AsRef<Path>) -> io::Result<std::fs::Metadata> {
    tokio::fs::symlink_metadata(path).await
}

#[cfg(target_arch = "wasm32")]
pub async fn symlink_metadata(path: impl AsRef<Path>) -> io::Result<std::fs::Metadata> {
    std::fs::symlink_metadata(path)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDir> {
    Ok(ReadDir {
        inner: tokio::fs::read_dir(path).await?,
    })
}

#[cfg(target_arch = "wasm32")]
pub async fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDir> {
    Ok(ReadDir {
        inner: std::fs::read_dir(path)?,
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub struct ReadDir {
    inner: tokio::fs::ReadDir,
}

#[cfg(target_arch = "wasm32")]
pub struct ReadDir {
    inner: std::fs::ReadDir,
}

#[cfg(not(target_arch = "wasm32"))]
impl ReadDir {
    pub async fn next_entry(&mut self) -> io::Result<Option<DirEntry>> {
        self.inner
            .next_entry()
            .await
            .map(|entry| entry.map(|inner| DirEntry { inner }))
    }
}

#[cfg(target_arch = "wasm32")]
impl ReadDir {
    pub async fn next_entry(&mut self) -> io::Result<Option<DirEntry>> {
        self.inner
            .next()
            .transpose()
            .map(|entry| entry.map(|inner| DirEntry { inner }))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct DirEntry {
    inner: tokio::fs::DirEntry,
}

#[cfg(target_arch = "wasm32")]
pub struct DirEntry {
    inner: std::fs::DirEntry,
}

#[cfg(not(target_arch = "wasm32"))]
impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.inner.path()
    }
}

#[cfg(target_arch = "wasm32")]
impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.inner.path()
    }
}
