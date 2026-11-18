use crate::models::{DownloadState, PackageItem, VersionManifest};
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use tar::Archive;
use thiserror::Error;
use tokio::sync::mpsc::Sender;

#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Archive error: {0}")]
    Archive(String),
    #[error("Checksum mismatch: expected {expected}, calculated {calculated}")]
    ChecksumMismatch { expected: String, calculated: String },
}

pub struct PackageManager;

impl PackageManager {
    /// Load manifest from a local file, or optionally try remote URL first
    pub async fn load_manifest(
        local_path: &Path,
        remote_url: Option<&str>,
    ) -> Result<VersionManifest, PackageError> {
        if let Some(url) = remote_url {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()?;
            
            if let Ok(resp) = client.get(url).send().await {
                if resp.status().is_success() {
                    if let Ok(manifest) = resp.json::<VersionManifest>().await {
                        // Cache remote manifest locally
                        if let Ok(json_str) = serde_json::to_string_pretty(&manifest) {
                            let _ = fs::write(local_path, json_str);
                        }
                        return Ok(manifest);
                    }
                }
            }
        }

        // Fallback to local manifest file
        let content = fs::read_to_string(local_path)?;
        let manifest: VersionManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    /// Download a package item and report progress via channel
    pub async fn download_and_extract(
        item: &PackageItem,
        destination_dir: &Path,
        progress_tx: Option<Sender<DownloadState>>,
    ) -> Result<PathBuf, PackageError> {
        let client = reqwest::Client::new();
        let resp = client.get(&item.url).send().await?;

        if !resp.status().is_success() {
            let msg = format!("HTTP {} ({})", resp.status(), item.url);
            if let Some(tx) = &progress_tx {
                let _ = tx.send(DownloadState::Failed(msg.clone())).await;
            }
            return Err(PackageError::Archive(msg));
        }

        let total_size = resp.content_length();
        let mut downloaded: u64 = 0;

        let temp_dir = tempfile::tempdir()?;
        let file_name = item
            .url
            .split('/')
            .last()
            .unwrap_or("package.archive")
            .split('?')
            .next()
            .unwrap_or("package.archive");
        let temp_file_path = temp_dir.path().join(file_name);

        let mut temp_file = tokio::fs::File::create(&temp_file_path).await?;
        let mut stream = resp.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            tokio::io::AsyncWriteExt::write_all(&mut temp_file, &chunk).await?;
            downloaded += chunk.len() as u64;

            if let Some(tx) = &progress_tx {
                let percent = if let Some(total) = total_size {
                    if total > 0 {
                        (downloaded as f32 / total as f32) * 100.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                let _ = tx
                    .send(DownloadState::Downloading {
                        downloaded_bytes: downloaded,
                        total_bytes: total_size,
                        percent,
                    })
                    .await;
            }
        }

        tokio::io::AsyncWriteExt::flush(&mut temp_file).await?;
        drop(temp_file);

        // Verify SHA256 checksum if provided
        if let Some(expected_hash) = &item.sha256 {
            let calculated = Self::compute_sha256(&temp_file_path)?;
            if !calculated.eq_ignore_ascii_case(expected_hash) {
                let err = PackageError::ChecksumMismatch {
                    expected: expected_hash.clone(),
                    calculated,
                };
                if let Some(tx) = &progress_tx {
                    let _ = tx.send(DownloadState::Failed(err.to_string())).await;
                }
                return Err(err);
            }
        }

        // Notify extracting state
        if let Some(tx) = &progress_tx {
            let _ = tx.send(DownloadState::Extracting).await;
        }

        // Target path
        let target_dir = destination_dir
            .join(item.service_type.folder_name())
            .join(&item.version);
        fs::create_dir_all(&target_dir)?;

        // Extract based on file extension
        if file_name.ends_with(".zip") {
            Self::extract_zip(&temp_file_path, &target_dir)?;
        } else if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            Self::extract_tar_gz(&temp_file_path, &target_dir)?;
        } else if file_name.ends_with(".tar.xz") || file_name.ends_with(".txz") {
            Self::extract_tar_xz(&temp_file_path, &target_dir)?;
        } else {
            return Err(PackageError::Archive(format!(
                "Unsupported archive format: {}",
                file_name
            )));
        }

        if let Some(tx) = &progress_tx {
            let _ = tx.send(DownloadState::Completed).await;
        }

        Ok(target_dir)
    }

    fn compute_sha256(path: &Path) -> Result<String, io::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    fn extract_zip(archive_path: &Path, destination: &Path) -> Result<(), PackageError> {
        let file = File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| PackageError::Archive(format!("Failed to read zip: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| PackageError::Archive(format!("Zip entry error: {}", e)))?;
            let outpath = match file.enclosed_name() {
                Some(path) => destination.join(path),
                None => continue,
            };

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }

            // On unix, restore file permissions if available
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    let _ = fs::set_permissions(&outpath, fs::Permissions::from_mode(mode));
                }
            }
        }

        Ok(())
    }

    fn extract_tar_gz(archive_path: &Path, destination: &Path) -> Result<(), PackageError> {
        let file = File::open(archive_path)?;
        let tar_gz = GzDecoder::new(file);
        let mut archive = Archive::new(tar_gz);
        archive
            .unpack(destination)
            .map_err(|e| PackageError::Archive(format!("Failed to unpack tar.gz: {}", e)))?;
        Ok(())
    }

    fn extract_tar_xz(archive_path: &Path, destination: &Path) -> Result<(), PackageError> {
        let status = std::process::Command::new("tar")
            .arg("-xf")
            .arg(archive_path)
            .arg("-C")
            .arg(destination)
            .status()
            .map_err(|e| PackageError::Archive(format!("Failed to execute tar: {}", e)))?;

        if !status.success() {
            return Err(PackageError::Archive(format!(
                "tar command exited with error status: {:?}",
                status.code()
            )));
        }
        Ok(())
    }
}
