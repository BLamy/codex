use std::io;
use std::path::Path;
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    tokio::fs::create_dir_all(path).await
}

#[cfg(target_arch = "wasm32")]
pub async fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    std::fs::create_dir_all(path)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    tokio::fs::write(path, contents).await
}

#[cfg(target_arch = "wasm32")]
pub async fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    std::fs::write(path, contents)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn write_new(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await?;
    file.write_all(contents.as_ref()).await
}

#[cfg(target_arch = "wasm32")]
pub async fn write_new(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    use std::io::Write as _;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(contents.as_ref())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    tokio::fs::remove_file(path).await
}

#[cfg(target_arch = "wasm32")]
pub async fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    std::fs::remove_file(path)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    tokio::fs::remove_dir_all(path).await
}

#[cfg(target_arch = "wasm32")]
pub async fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    std::fs::remove_dir_all(path)
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
pub async fn try_exists(path: impl AsRef<Path>) -> io::Result<bool> {
    tokio::fs::try_exists(path).await
}

#[cfg(target_arch = "wasm32")]
pub async fn try_exists(path: impl AsRef<Path>) -> io::Result<bool> {
    Ok(path.as_ref().try_exists()?)
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

    pub async fn file_type(&self) -> io::Result<std::fs::FileType> {
        self.inner.file_type().await
    }
}

#[cfg(target_arch = "wasm32")]
impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.inner.path()
    }

    pub async fn file_type(&self) -> io::Result<std::fs::FileType> {
        self.inner.file_type()
    }
}
