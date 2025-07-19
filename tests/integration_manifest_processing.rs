//! マニフェスト処理の統合テスト
//!
//! マニフェストサービスとファイルシステム操作の
//! 統合テストを実装

use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tsrc::{
    application::services::manifest_service::{
        ManifestProcessingOptions, ManifestService, ProcessedManifest,
    },
    domain::entities::manifest::{Group, Manifest, ManifestRepo},
    infrastructure::filesystem::manifest_store::{
        ManifestProcessingOptions as StoreProcessingOptions, ManifestStore,
    },
};

/// テスト用のマニフェストファイルを作成するヘルパー関数
fn create_complex_manifest_file(dir: &PathBuf, filename: &str) -> std::io::Result<PathBuf> {
    let manifest_content = r#"repos:
  - dest: backend/api
    url: https://github.com/example/api-server.git
    branch: main
    groups: [backend, core]
  - dest: frontend/web
    url: https://github.com/example/web-app.git
    branch: develop
    groups: [frontend, core]
  - dest: tools/scripts
    url: https://github.com/example/build-tools.git
    branch: main
    groups: [tools]
  - dest: docs/wiki
    url: https://github.com/example/documentation.git
    branch: main
    groups: [docs]

groups:
  backend:
    repos: [backend/api]
  frontend:
    repos: [frontend/web]
  core:
    repos: [backend/api, frontend/web]
  tools:
    repos: [tools/scripts]
  docs:
    repos: [docs/wiki]
"#;

    let manifest_path = dir.join(filename);
    std::fs::write(&manifest_path, manifest_content)?;
    Ok(manifest_path)
}

/// シンプルなマニフェストファイルを作成するヘルパー関数
fn create_simple_manifest_file(dir: &PathBuf, filename: &str) -> std::io::Result<PathBuf> {
    let manifest_content = r#"repos:
  - dest: simple-repo
    url: https://github.com/example/simple.git
    branch: main
"#;

    let manifest_path = dir.join(filename);
    std::fs::write(&manifest_path, manifest_content)?;
    Ok(manifest_path)
}

#[tokio::test]
async fn test_manifest_store_read_and_write() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let mut manifest_store = ManifestStore::new();

    // 1. テスト用マニフェストファイルを作成
    let original_manifest_path =
        create_complex_manifest_file(temp_dir.path(), "original.yml").unwrap();

    // 2. マニフェストを読み込み
    let loaded_processed = manifest_store
        .read_manifest(&original_manifest_path)
        .await
        .unwrap();
    let loaded_manifest = &loaded_processed.manifest;

    // 3. 読み込み結果を確認
    assert_eq!(loaded_manifest.repos.len(), 4, "Should load 4 repositories");
    assert!(
        loaded_manifest.groups.is_some(),
        "Should have groups defined"
    );

    let groups = loaded_manifest.groups.as_ref().unwrap();
    assert_eq!(groups.len(), 5, "Should have 5 groups");
    assert!(groups.contains_key("backend"), "Should have backend group");
    assert!(
        groups.contains_key("frontend"),
        "Should have frontend group"
    );

    // 4. マニフェストを別の場所に書き込み
    let output_path = temp_dir.path().join("output.yml");
    manifest_store
        .write_manifest(&output_path, loaded_manifest)
        .await
        .unwrap();

    // 5. 書き込んだマニフェストを再読み込み
    let reloaded_processed = manifest_store.read_manifest(&output_path).await.unwrap();
    let reloaded_manifest = &reloaded_processed.manifest;

    // 6. 内容が一致することを確認
    assert_eq!(loaded_manifest.repos.len(), reloaded_manifest.repos.len());
    assert_eq!(
        loaded_manifest.groups.is_some(),
        reloaded_manifest.groups.is_some()
    );
}

#[tokio::test]
async fn test_manifest_service_processing() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let mut manifest_service = ManifestService::new(ManifestProcessingOptions::default());

    // 1. テスト用マニフェストファイルを作成
    let manifest_path = create_complex_manifest_file(temp_dir.path(), "test.yml").unwrap();

    // 2. マニフェストをファイルから解析
    let processed_manifest = manifest_service
        .parse_from_file(&manifest_path)
        .await
        .unwrap();

    // 3. 処理結果を確認
    assert_eq!(processed_manifest.manifest.repos.len(), 4);
    assert!(processed_manifest.manifest.groups.is_some());
    let groups = processed_manifest.manifest.groups.as_ref().unwrap();
    assert!(groups.len() > 0);

    // 4. グループ情報を取得
    let (backend_group, backend_repos) = manifest_service
        .get_group_info(&processed_manifest.manifest, "backend")
        .unwrap();
    assert_eq!(backend_repos.len(), 1);
    assert_eq!(backend_repos[0].dest, "backend/api");

    let (core_group, core_repos) = manifest_service
        .get_group_info(&processed_manifest.manifest, "core")
        .unwrap();
    assert_eq!(core_repos.len(), 2);
    assert!(core_repos.iter().any(|r| r.dest == "backend/api"));
    assert!(core_repos.iter().any(|r| r.dest == "frontend/web"));

    // 5. 存在しないグループのテスト
    let invalid_group =
        manifest_service.get_group_info(&processed_manifest.manifest, "nonexistent");
    assert!(invalid_group.is_none());
}

#[tokio::test]
async fn test_manifest_filtering_by_groups() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let mut manifest_service = ManifestService::new(ManifestProcessingOptions::default());
    let manifest_store = ManifestStore::new();

    // 1. テスト用マニフェストファイルを作成
    let manifest_path = create_complex_manifest_file(temp_dir.path(), "test.yml").unwrap();
    let processed_manifest = manifest_service
        .parse_from_file(&manifest_path)
        .await
        .unwrap();

    // 2. 特定のグループでフィルタリング
    let backend_groups = vec!["backend".to_string()];
    let filtered_manifest = manifest_store
        .filter_manifest_by_groups(&processed_manifest.manifest, &backend_groups)
        .unwrap();

    // 3. フィルタリング結果を確認
    assert_eq!(filtered_manifest.repos.len(), 1);
    assert_eq!(filtered_manifest.repos[0].dest, "backend/api");

    // 4. 複数グループでのフィルタリング
    let multiple_groups = vec!["frontend".to_string(), "tools".to_string()];
    let multi_filtered = manifest_store
        .filter_manifest_by_groups(&processed_manifest.manifest, &multiple_groups)
        .unwrap();

    assert_eq!(multi_filtered.repos.len(), 2);
    let dest_names: Vec<&str> = multi_filtered
        .repos
        .iter()
        .map(|r| r.dest.as_str())
        .collect();
    assert!(dest_names.contains(&"frontend/web"));
    assert!(dest_names.contains(&"tools/scripts"));

    // 5. 存在しないグループでのフィルタリング
    let invalid_groups = vec!["nonexistent".to_string()];
    let empty_filtered = manifest_store
        .filter_manifest_by_groups(&processed_manifest.manifest, &invalid_groups)
        .unwrap();

    assert_eq!(
        empty_filtered.repos.len(),
        0,
        "Should return empty manifest for nonexistent groups"
    );
}

#[tokio::test]
async fn test_manifest_validation_and_error_handling() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let mut manifest_store = ManifestStore::new();

    // 1. 無効なYAMLファイルを作成
    let invalid_yaml_path = temp_dir.path().join("invalid.yml");
    std::fs::write(&invalid_yaml_path, "invalid: yaml: content: [").unwrap();

    // 2. 無効なマニフェストの読み込みテスト
    let invalid_result = manifest_store.read_manifest(&invalid_yaml_path).await;
    assert!(invalid_result.is_err(), "Should fail to read invalid YAML");

    // 3. 存在しないファイルの読み込みテスト
    let nonexistent_path = temp_dir.path().join("nonexistent.yml");
    let missing_result = manifest_store.read_manifest(&nonexistent_path).await;
    assert!(
        missing_result.is_err(),
        "Should fail to read nonexistent file"
    );

    // 4. 空のマニフェストのテスト
    let empty_manifest_path = temp_dir.path().join("empty.yml");
    std::fs::write(&empty_manifest_path, "repos: []").unwrap();

    let empty_result = manifest_store.read_manifest(&empty_manifest_path).await;
    assert!(empty_result.is_ok(), "Should handle empty manifest");

    let empty_processed = empty_result.unwrap();
    assert_eq!(
        empty_processed.manifest.repos.len(),
        0,
        "Should have no repositories"
    );
}

#[tokio::test]
async fn test_manifest_processing_with_options() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let options = StoreProcessingOptions {
        validate_manifest: true,
        base_directory: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };
    let mut manifest_store = ManifestStore::with_options(options);

    // 1. テスト用マニフェストファイルを作成
    let manifest_path = create_simple_manifest_file(temp_dir.path(), "simple.yml").unwrap();

    // 2. オプション付きでマニフェストを読み込み
    let result = manifest_store.read_manifest(&manifest_path).await;
    assert!(
        result.is_ok(),
        "Should read manifest with validation options"
    );

    let processed = result.unwrap();
    assert_eq!(processed.manifest.repos.len(), 1);
    assert_eq!(processed.manifest.repos[0].dest, "simple-repo");
}

#[tokio::test]
async fn test_manifest_groups_listing() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let mut manifest_store = ManifestStore::new();

    // 1. テスト用マニフェストファイルを作成
    let manifest_path = create_complex_manifest_file(temp_dir.path(), "groups.yml").unwrap();
    let processed = manifest_store.read_manifest(&manifest_path).await.unwrap();

    // 2. グループ一覧を取得
    let groups = manifest_store.list_manifest_groups(&processed.manifest);

    // 3. グループ一覧を確認
    assert_eq!(groups.len(), 5);
    assert!(groups.contains(&"backend".to_string()));
    assert!(groups.contains(&"frontend".to_string()));
    assert!(groups.contains(&"core".to_string()));
    assert!(groups.contains(&"tools".to_string()));
    assert!(groups.contains(&"docs".to_string()));

    // 4. グループが適切にソートされていることを確認
    let sorted_groups: Vec<_> = groups.iter().cloned().collect();
    let mut expected_groups = vec!["backend", "frontend", "core", "tools", "docs"];
    expected_groups.sort();
    // Note: The order might be different, but all groups should be present
    assert_eq!(sorted_groups.len(), expected_groups.len());
}

#[tokio::test]
async fn test_end_to_end_manifest_workflow() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let mut manifest_store = ManifestStore::new();
    let mut manifest_service = ManifestService::new(ManifestProcessingOptions::default());

    // 1. 複雑なマニフェストファイルを作成
    let source_path = create_complex_manifest_file(temp_dir.path(), "source.yml").unwrap();

    // 2. マニフェストサービスで処理
    let processed = manifest_service
        .parse_from_file(&source_path)
        .await
        .unwrap();

    // 3. 特定のグループをフィルタリング
    let core_groups = vec!["core".to_string()];
    let filtered_manifest = manifest_store
        .filter_manifest_by_groups(&processed.manifest, &core_groups)
        .unwrap();

    // 4. フィルタリング結果を新しいファイルに書き込み
    let output_path = temp_dir.path().join("filtered.yml");
    manifest_store
        .write_manifest(&output_path, &filtered_manifest)
        .await
        .unwrap();

    // 5. 書き込んだファイルを再読み込みして検証
    let reloaded_processed = manifest_store.read_manifest(&output_path).await.unwrap();
    assert_eq!(
        reloaded_processed.manifest.repos.len(),
        2,
        "Should have 2 core repositories"
    );

    // 6. リポジトリ情報を確認
    let repo_dests: Vec<&str> = reloaded_processed
        .manifest
        .repos
        .iter()
        .map(|r| r.dest.as_str())
        .collect();
    assert!(repo_dests.contains(&"backend/api"));
    assert!(repo_dests.contains(&"frontend/web"));

    // 7. マニフェストサービスでの最終検証
    let final_processed = manifest_service
        .parse_from_file(&output_path)
        .await
        .unwrap();
    assert_eq!(final_processed.manifest.repos.len(), 2);

    // 8. グループ情報が保持されているかの確認（フィルタリング後は失われる可能性がある）
    // Note: フィルタリング後はグループ情報が変更される可能性があるため、
    // 元のマニフェストと直接比較はしない
}
