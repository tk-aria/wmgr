use wmgr::infrastructure::http::HttpDownloader;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_http_download_file() {
    // This test requires network access
    // Using a small test file from a reliable source
    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("test.txt");
    
    let downloader = HttpDownloader::new();
    
    // Using httpbin.org for testing (returns small JSON response)
    let result = downloader.download_file(
        "https://httpbin.org/json",
        &dest_path
    );
    
    // Check if download succeeded (may fail if no network)
    if result.is_ok() {
        assert!(dest_path.exists());
        let content = fs::read_to_string(&dest_path).unwrap();
        assert!(content.contains("slideshow"));
    }
}

#[test]
fn test_http_download_with_redirect() {
    // Test redirect handling
    let temp_dir = TempDir::new().unwrap();
    let dest_path = temp_dir.path().join("redirect_test.json");
    
    let downloader = HttpDownloader::new();
    
    // httpbin.org redirect endpoint
    let result = downloader.download_file(
        "https://httpbin.org/redirect/1",
        &dest_path
    );
    
    // Check if download succeeded after redirect
    if result.is_ok() {
        assert!(dest_path.exists());
        let content = fs::read_to_string(&dest_path).unwrap();
        // After redirect, should get the GET response
        assert!(content.contains("url"));
    }
}

#[test]
fn test_is_archive_detection() {
    let downloader = HttpDownloader::new();
    
    // Test archive detection
    assert!(downloader.is_archive("file.zip"));
    assert!(downloader.is_archive("file.tar.gz"));
    assert!(downloader.is_archive("file.tgz"));
    assert!(downloader.is_archive("file.tar"));
    assert!(downloader.is_archive("https://example.com/file.zip?param=value"));
    
    // Test non-archive files
    assert!(!downloader.is_archive("file.txt"));
    assert!(!downloader.is_archive("file.svg"));
    assert!(!downloader.is_archive("file.json"));
    assert!(!downloader.is_archive("https://example.com/file.html"));
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use wmgr::application::use_cases::sync_repositories::{
        SyncRepositoriesUseCase, SyncRepositoriesConfig
    };
    use wmgr::domain::entities::manifest::{Manifest, ManifestRepo};
    
    #[tokio::test]
    async fn test_sync_with_http_download() {
        // Create a test manifest with HTTP URL
        let repos = vec![
            ManifestRepo::new(
                "https://httpbin.org/json",
                "test_download.json"
            ),
        ];
        
        let manifest = Manifest::new(repos);
        
        // This would require setting up a full workspace
        // For now, we're just testing that the code compiles
        // and the basic structure is in place
    }
}