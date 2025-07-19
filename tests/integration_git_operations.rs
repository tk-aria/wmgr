//! Git操作の統合テスト
//!
//! GitRepository、GitRemoteManager、および関連する
//! インフラストラクチャ層の統合テストを実装

use std::path::PathBuf;
use tempfile::TempDir;
use tsrc::{
    domain::{
        entities::repository::{Remote, Repository},
        value_objects::{branch_name::BranchName, git_url::GitUrl},
    },
    infrastructure::git::{
        remote::{GitRemoteManager, RemoteInfo},
        repository::{CloneConfig, FetchConfig, GitBranchType, GitRepository, ResetMode},
    },
};

/// テスト用のGitリポジトリを初期化するヘルパー関数
fn init_test_repository(dir: &PathBuf) -> std::io::Result<()> {
    // 簡易的なGitリポジトリ構造を作成
    std::fs::create_dir_all(dir.join(".git"))?;
    std::fs::write(dir.join(".git").join("HEAD"), "ref: refs/heads/main\n")?;
    std::fs::create_dir_all(dir.join(".git").join("refs").join("heads"))?;
    std::fs::write(
        dir.join(".git").join("refs").join("heads").join("main"),
        "dummy-commit-hash\n",
    )?;

    // テスト用ファイルを追加
    std::fs::write(dir.join("README.md"), "# Test Repository\n")?;
    std::fs::write(
        dir.join("src").join("main.rs"),
        "fn main() { println!(\"Hello, world!\"); }\n",
    )?;

    Ok(())
}

/// テスト用のリモートリポジトリ情報を作成するヘルパー関数
fn create_test_remotes() -> Vec<Remote> {
    vec![
        Remote::new("origin", "https://github.com/example/repo.git"),
        Remote::new("upstream", "https://github.com/upstream/repo.git"),
    ]
}

#[tokio::test]
async fn test_git_repository_initialization() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo");
    std::fs::create_dir_all(&repo_path).unwrap();

    // 1. 新しいGitリポジトリを初期化
    let result = GitRepository::init(&repo_path);

    // 初期化の成功を確認
    assert!(result.is_ok(), "Repository initialization should succeed");
    let git_repo = result.unwrap();

    // 2. .gitディレクトリが作成されていることを確認
    let git_dir = repo_path.join(".git");
    assert!(git_dir.exists(), ".git directory should be created");

    // 3. リポジトリが有効であることを確認
    assert!(
        git_repo.is_valid(),
        "Repository should be valid after initialization"
    );

    // 4. 現在のブランチを確認
    let current_branch = git_repo.current_branch();
    assert!(
        current_branch.is_ok(),
        "Should be able to get current branch"
    );
}

#[tokio::test]
async fn test_git_repository_opening() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("existing-repo");
    std::fs::create_dir_all(&repo_path).unwrap();
    init_test_repository(&repo_path).unwrap();

    // 1. 既存のリポジトリを開く
    let result = GitRepository::open(&repo_path);
    assert!(result.is_ok(), "Should be able to open existing repository");

    let git_repo = result.unwrap();

    // 2. リポジトリの基本情報を確認
    assert!(git_repo.is_valid(), "Opened repository should be valid");

    let current_branch = git_repo.current_branch();
    assert!(
        current_branch.is_ok(),
        "Should be able to get current branch"
    );

    // 3. 存在しないリポジトリを開こうとする場合のテスト
    let nonexistent_path = temp_dir.path().join("nonexistent");
    let invalid_result = GitRepository::open(&nonexistent_path);
    assert!(
        invalid_result.is_err(),
        "Should fail to open nonexistent repository"
    );
}

#[tokio::test]
async fn test_git_repository_branch_operations() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("branch-test");
    std::fs::create_dir_all(&repo_path).unwrap();

    // リポジトリを初期化
    let git_repo = GitRepository::init(&repo_path).unwrap();

    // 1. 現在のブランチを取得
    let current_branch = git_repo.current_branch();
    assert!(
        current_branch.is_ok(),
        "Should be able to get current branch"
    );

    // 2. ブランチ一覧を取得
    let branches = git_repo.list_branches(Some(GitBranchType::Local));
    assert!(branches.is_ok(), "Should be able to list branches");

    // 3. 新しいブランチを作成
    let new_branch = BranchName::new("feature/test-branch").unwrap();
    let create_result = git_repo.create_branch(&new_branch, None);
    assert!(create_result.is_ok(), "Should be able to create new branch");

    // 4. ブランチをチェックアウト
    let checkout_result = git_repo.checkout_branch(&new_branch);
    assert!(checkout_result.is_ok(), "Should be able to checkout branch");
}

#[tokio::test]
async fn test_git_remote_manager_operations() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("remote-test");
    std::fs::create_dir_all(&repo_path).unwrap();

    // リポジトリを初期化
    let git_repo = GitRepository::init(&repo_path).unwrap();

    // 1. GitRemoteManagerを作成
    let remote_manager = GitRemoteManager::new(&git_repo);
    assert!(
        remote_manager.is_ok(),
        "Should be able to create remote manager"
    );

    let manager = remote_manager.unwrap();

    // 2. リモートを追加
    let origin_url = GitUrl::new("https://github.com/example/repo.git").unwrap();
    let add_result = manager.add_remote("origin", &origin_url);
    assert!(add_result.is_ok(), "Should be able to add remote");

    // 3. リモート一覧を取得
    let remotes = manager.list_remotes();
    assert!(remotes.is_ok(), "Should be able to list remotes");

    let remote_list = remotes.unwrap();
    assert_eq!(remote_list.len(), 1, "Should have one remote");
    assert_eq!(
        remote_list[0].name, "origin",
        "Remote name should be 'origin'"
    );

    // 4. リモート情報を取得
    let remote_info = manager.get_remote_info("origin");
    assert!(remote_info.is_ok(), "Should be able to get remote info");

    let info = remote_info.unwrap();
    assert_eq!(info.name, "origin");
    assert_eq!(
        info.fetch_url.as_str(),
        "https://github.com/example/repo.git"
    );

    // 5. 追加のリモートを設定
    let upstream_url = GitUrl::new("https://github.com/upstream/repo.git").unwrap();
    let upstream_result = manager.add_remote("upstream", &upstream_url);
    assert!(
        upstream_result.is_ok(),
        "Should be able to add upstream remote"
    );

    // 6. 複数リモートの確認
    let all_remotes = manager.list_remotes().unwrap();
    assert_eq!(all_remotes.len(), 2, "Should have two remotes");
}

#[tokio::test]
async fn test_domain_repository_entity_integration() {
    // テスト環境の準備
    let remotes = create_test_remotes();

    // 1. Repositoryエンティティを作成
    let repository = Repository::new("test/repo", remotes.clone());

    // 2. エンティティの基本情報を確認
    assert_eq!(repository.dest, "test/repo");
    assert_eq!(repository.remotes.len(), 2);

    // 3. 特定のリモートを取得
    let origin_remote = repository.get_remote("origin");
    assert!(origin_remote.is_some(), "Should find origin remote");
    assert_eq!(
        origin_remote.unwrap().url,
        "https://github.com/example/repo.git"
    );

    let upstream_remote = repository.get_remote("upstream");
    assert!(upstream_remote.is_some(), "Should find upstream remote");

    let nonexistent_remote = repository.get_remote("nonexistent");
    assert!(
        nonexistent_remote.is_none(),
        "Should not find nonexistent remote"
    );

    // 4. Repositoryのbuilder機能をテスト
    let built_repo = Repository::builder("built/repo")
        .with_remote(Remote::new("origin", "https://github.com/test/repo.git"))
        .with_remote(Remote::new("backup", "https://backup.com/test/repo.git"))
        .build();

    assert_eq!(built_repo.dest, "built/repo");
    assert_eq!(built_repo.remotes.len(), 2);
    assert!(built_repo.get_remote("origin").is_some());
    assert!(built_repo.get_remote("backup").is_some());
}

#[tokio::test]
async fn test_git_url_and_repository_integration() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("url-test");
    std::fs::create_dir_all(&repo_path).unwrap();

    // 1. 様々なURL形式をテスト
    let https_url = GitUrl::new("https://github.com/example/repo.git").unwrap();
    let ssh_url = GitUrl::new("git@github.com:example/repo.git").unwrap();

    // 2. URLの等価性を確認
    assert!(
        https_url.is_same_repo(&ssh_url),
        "HTTPS and SSH URLs should point to same repo"
    );

    // 3. URL変換をテスト
    assert_eq!(https_url.to_ssh_url(), "git@github.com:example/repo.git");
    assert_eq!(
        ssh_url.to_https_url(),
        "https://github.com/example/repo.git"
    );

    // 4. Gitリポジトリでの利用をテスト
    let git_repo = GitRepository::init(&repo_path).unwrap();
    let manager = GitRemoteManager::new(&git_repo).unwrap();

    // HTTPS URLでリモートを追加
    let https_result = manager.add_remote("https-origin", &https_url);
    assert!(https_result.is_ok(), "Should be able to add HTTPS remote");

    // SSH URLでリモートを追加
    let ssh_result = manager.add_remote("ssh-origin", &ssh_url);
    assert!(ssh_result.is_ok(), "Should be able to add SSH remote");

    // 5. リモート情報を確認
    let remotes = manager.list_remotes().unwrap();
    assert_eq!(remotes.len(), 2, "Should have two remotes");
}

#[tokio::test]
async fn test_git_operations_error_handling() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();

    // 1. 無効なパスでのリポジトリ操作
    let invalid_path = temp_dir.path().join("invalid\0path");
    let invalid_result = GitRepository::init(&invalid_path);
    assert!(invalid_result.is_err(), "Should fail with invalid path");

    // 2. 既存ファイルと同じ名前でのリポジトリ初期化
    let file_path = temp_dir.path().join("existing-file");
    std::fs::write(&file_path, "content").unwrap();
    let file_result = GitRepository::init(&file_path);
    assert!(
        file_result.is_err(),
        "Should fail when path is an existing file"
    );

    // 3. 無効なURLでのリモート追加
    let repo_path = temp_dir.path().join("error-test");
    let git_repo = GitRepository::init(&repo_path).unwrap();
    let manager = GitRemoteManager::new(&git_repo).unwrap();

    let invalid_url_result = GitUrl::new("invalid-url");
    assert!(
        invalid_url_result.is_err(),
        "Should fail to parse invalid URL"
    );

    // 4. 重複するリモート名での追加
    let valid_url = GitUrl::new("https://github.com/example/repo.git").unwrap();
    manager.add_remote("origin", &valid_url).unwrap();

    let duplicate_result = manager.add_remote("origin", &valid_url);
    assert!(
        duplicate_result.is_err(),
        "Should fail to add duplicate remote"
    );
}

#[tokio::test]
async fn test_comprehensive_git_workflow() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("workflow-test");
    std::fs::create_dir_all(&repo_path).unwrap();

    // 1. リポジトリ初期化
    let git_repo = GitRepository::init(&repo_path).unwrap();
    assert!(git_repo.is_valid());

    // 2. リモート設定
    let manager = GitRemoteManager::new(&git_repo).unwrap();
    let origin_url = GitUrl::new("https://github.com/example/repo.git").unwrap();
    manager.add_remote("origin", &origin_url).unwrap();

    // 3. ブランチ作成とチェックアウト
    let feature_branch = BranchName::new("feature/integration-test").unwrap();
    git_repo.create_branch(&feature_branch, None).unwrap();
    git_repo.checkout_branch(&feature_branch).unwrap();

    // 4. ファイル変更をシミュレート
    std::fs::write(repo_path.join("test.txt"), "integration test content").unwrap();

    // 5. リポジトリステータスを確認
    let status = git_repo.status();
    assert!(status.is_ok(), "Should be able to get repository status");

    // 6. ブランチ一覧を確認
    let branches = git_repo.list_branches(Some(GitBranchType::Local)).unwrap();
    assert!(
        branches.len() >= 2,
        "Should have at least main and feature branch"
    );

    // 7. リモート情報を最終確認
    let remotes = manager.list_remotes().unwrap();
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0].name, "origin");

    // 8. Domain層エンティティとの統合確認
    let domain_remotes = vec![Remote::new("origin", origin_url.as_str())];
    let repository = Repository::new(
        &repo_path.file_name().unwrap().to_str().unwrap(),
        domain_remotes,
    );

    assert_eq!(repository.dest, "workflow-test");
    assert_eq!(repository.remotes.len(), 1);
    assert!(repository.get_remote("origin").is_some());
}
