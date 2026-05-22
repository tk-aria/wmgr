use crate::common::error::WmgrError;
use crate::common::result::WmgrResult;
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tar::Archive;
use tracing::{debug, info, warn};
use zip::ZipArchive;

/// HTTP download manager for handling file downloads and extraction
pub struct HttpDownloader {
    client: Client,
}

impl HttpDownloader {
    /// Create a new HTTP downloader with redirect support
    pub fn new() -> Self {
        let client = Client::builder()
            .redirect(Policy::limited(10)) // Follow up to 10 redirects
            .timeout(Duration::from_secs(300)) // 5 minutes timeout
            .user_agent("wmgr/1.0")
            .build()
            .unwrap_or_else(|_| Client::new());
            
        Self { client }
    }

    /// Download a file from URL and optionally extract if it's an archive
    pub fn download_and_extract(&self, url: &str, dest_path: &Path) -> WmgrResult<()> {
        info!("Downloading from URL: {}", url);
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                WmgrError::IoError(format!("Failed to create parent directory: {}", e))
            })?;
        }

        // Download the file
        let response = self.client.get(url).send().map_err(|e| {
            WmgrError::network_error_with_source(
                format!("Failed to download from {}", url),
                Some(url.to_string()),
                e
            )
        })?;

        // Log redirect information if URL changed
        let final_url = response.url().clone();
        let final_url_str = final_url.as_str();
        if final_url_str != url {
            info!("Redirected to: {}", final_url_str);
        }

        if !response.status().is_success() {
            return Err(WmgrError::network_error(
                format!("HTTP request failed with status: {}", response.status()),
                Some(final_url_str.to_string())
            ));
        }

        // Get file extension from URL to determine if extraction is needed
        // Check both original and final URL (after redirect) for archive detection
        let original_url_path = url.split('?').next().unwrap_or(url);
        let final_url_path = final_url_str.split('?').next().unwrap_or(final_url_str);
        let needs_extraction = self.is_archive(original_url_path) || self.is_archive(final_url_path);

        if needs_extraction {
            // Download to temporary file and extract
            let temp_file = tempfile::NamedTempFile::new().map_err(|e| {
                WmgrError::IoError(format!("Failed to create temp file: {}", e))
            })?;

            let mut temp_file_write = temp_file.reopen().map_err(|e| {
                WmgrError::IoError(format!("Failed to open temp file for writing: {}", e))
            })?;

            let content = response.bytes().map_err(|e| {
                WmgrError::network_error_with_source(
                    "Failed to read response body".to_string(),
                    Some(url.to_string()),
                    e
                )
            })?;

            temp_file_write.write_all(&content).map_err(|e| {
                WmgrError::IoError(format!("Failed to write to temp file: {}", e))
            })?;
            
            temp_file_write.flush().map_err(|e| {
                WmgrError::IoError(format!("Failed to flush temp file: {}", e))
            })?;

            drop(temp_file_write);

            // Extract based on file type (use final URL for detection)
            self.extract_archive(temp_file.path(), final_url_path, dest_path)?;
            
            info!("Successfully extracted archive to: {}", dest_path.display());
        } else {
            // Direct file download
            let content = response.bytes().map_err(|e| {
                WmgrError::network_error_with_source(
                    "Failed to read response body".to_string(),
                    Some(url.to_string()),
                    e
                )
            })?;

            let mut file = File::create(dest_path).map_err(|e| {
                WmgrError::IoError(format!("Failed to create file {}: {}", dest_path.display(), e))
            })?;

            file.write_all(&content).map_err(|e| {
                WmgrError::IoError(format!("Failed to write file {}: {}", dest_path.display(), e))
            })?;

            info!("Successfully downloaded file to: {}", dest_path.display());
        }

        Ok(())
    }

    /// Check if the URL points to an archive file
    pub fn is_archive(&self, url_path: &str) -> bool {
        // Remove query parameters if present
        let path = url_path.split('?').next().unwrap_or(url_path);
        
        path.ends_with(".zip")
            || path.ends_with(".tar")
            || path.ends_with(".tar.gz")
            || path.ends_with(".tgz")
            || path.ends_with(".tar.bz2")
            || path.ends_with(".tbz2")
    }

    /// Extract archive based on its type
    fn extract_archive(&self, archive_path: &Path, url_path: &str, dest_path: &Path) -> WmgrResult<()> {
        debug!("Extracting archive: {} to {}", archive_path.display(), dest_path.display());

        // Create destination directory
        fs::create_dir_all(dest_path).map_err(|e| {
            WmgrError::IoError(format!("Failed to create destination directory: {}", e))
        })?;

        if url_path.ends_with(".zip") {
            self.extract_zip(archive_path, dest_path)?;
        } else if url_path.ends_with(".tar.gz") || url_path.ends_with(".tgz") {
            self.extract_tar_gz(archive_path, dest_path)?;
        } else if url_path.ends_with(".tar") {
            self.extract_tar(archive_path, dest_path)?;
        } else if url_path.ends_with(".tar.bz2") || url_path.ends_with(".tbz2") {
            return Err(WmgrError::UnsupportedOperation(
                "bzip2 archives are not yet supported".to_string()
            ));
        }

        Ok(())
    }

    /// Extract ZIP archive
    fn extract_zip(&self, archive_path: &Path, dest_path: &Path) -> WmgrResult<()> {
        let file = File::open(archive_path).map_err(|e| {
            WmgrError::IoError(format!("Failed to open zip file: {}", e))
        })?;

        let mut archive = ZipArchive::new(file).map_err(|e| {
            WmgrError::IoError(format!("Failed to read zip archive: {}", e))
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                WmgrError::IoError(format!("Failed to read zip entry: {}", e))
            })?;

            let outpath = match file.enclosed_name() {
                Some(path) => dest_path.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| {
                    WmgrError::IoError(format!("Failed to create directory: {}", e))
                })?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| {
                            WmgrError::IoError(format!("Failed to create parent directory: {}", e))
                        })?;
                    }
                }
                let mut outfile = File::create(&outpath).map_err(|e| {
                    WmgrError::IoError(format!("Failed to create file: {}", e))
                })?;
                io::copy(&mut file, &mut outfile).map_err(|e| {
                    WmgrError::IoError(format!("Failed to extract file: {}", e))
                })?;
            }

            // Set permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).ok();
                }
            }
        }

        Ok(())
    }

    /// Extract tar.gz archive
    fn extract_tar_gz(&self, archive_path: &Path, dest_path: &Path) -> WmgrResult<()> {
        let tar_gz = File::open(archive_path).map_err(|e| {
            WmgrError::IoError(format!("Failed to open tar.gz file: {}", e))
        })?;

        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        archive.unpack(dest_path).map_err(|e| {
            WmgrError::IoError(format!("Failed to extract tar.gz archive: {}", e))
        })?;

        Ok(())
    }

    /// Extract tar archive
    fn extract_tar(&self, archive_path: &Path, dest_path: &Path) -> WmgrResult<()> {
        let tar = File::open(archive_path).map_err(|e| {
            WmgrError::IoError(format!("Failed to open tar file: {}", e))
        })?;

        let mut archive = Archive::new(tar);

        archive.unpack(dest_path).map_err(|e| {
            WmgrError::IoError(format!("Failed to extract tar archive: {}", e))
        })?;

        Ok(())
    }

    /// Download file without extraction
    pub fn download_file(&self, url: &str, dest_path: &Path) -> WmgrResult<()> {
        info!("Downloading file from URL: {}", url);

        // Create parent directory if it doesn't exist
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                WmgrError::IoError(format!("Failed to create parent directory: {}", e))
            })?;
        }

        let response = self.client.get(url).send().map_err(|e| {
            WmgrError::network_error_with_source(
                format!("Failed to download from {}", url),
                Some(url.to_string()),
                e
            )
        })?;

        // Log redirect information if URL changed
        let final_url = response.url().clone();
        let final_url_str = final_url.as_str();
        if final_url_str != url {
            info!("Redirected to: {}", final_url_str);
        }

        if !response.status().is_success() {
            return Err(WmgrError::network_error(
                format!("HTTP request failed with status: {}", response.status()),
                Some(final_url_str.to_string())
            ));
        }

        let content = response.bytes().map_err(|e| {
            WmgrError::network_error_with_source(
                "Failed to read response body".to_string(),
                Some(url.to_string()),
                e
            )
        })?;

        let mut file = File::create(dest_path).map_err(|e| {
            WmgrError::IoError(format!("Failed to create file {}: {}", dest_path.display(), e))
        })?;

        file.write_all(&content).map_err(|e| {
            WmgrError::IoError(format!("Failed to write file {}: {}", dest_path.display(), e))
        })?;

        info!("Successfully downloaded file to: {}", dest_path.display());
        Ok(())
    }
}

impl Default for HttpDownloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_archive() {
        let downloader = HttpDownloader::new();
        
        assert!(downloader.is_archive("file.zip"));
        assert!(downloader.is_archive("file.tar.gz"));
        assert!(downloader.is_archive("file.tgz"));
        assert!(downloader.is_archive("https://example.com/file.zip?param=value"));
        assert!(!downloader.is_archive("file.txt"));
        assert!(!downloader.is_archive("file.svg"));
    }
}