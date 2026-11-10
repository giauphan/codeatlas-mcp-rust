use std::path::Path;

#[cfg(unix)]
pub async fn create_secure_dir_all(dir: &Path) -> std::io::Result<()> {
    let mut builder = tokio::fs::DirBuilder::new();
    builder.recursive(true).mode(0o700);
    builder.create(dir).await
}

#[cfg(not(unix))]
pub async fn create_secure_dir_all(dir: &Path) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dir).await
}

#[cfg(unix)]
pub async fn write_secure_file(path: &Path, content: &[u8]) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt;
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true).mode(0o600);
    let mut file = options.open(path).await?;
    file.write_all(content).await
}

#[cfg(not(unix))]
pub async fn write_secure_file(path: &Path, content: &[u8]) -> std::io::Result<()> {
    tokio::fs::write(path, content).await
}
