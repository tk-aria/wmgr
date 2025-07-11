use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::domain::entities::{workspace::{Workspace, WorkspaceConfig}, manifest::Manifest};
use crate::domain::value_objects::{git_url::GitUrl, file_path::FilePath};

/// InitWorkspace関連のエラー
#[derive(Debug, Error)]
pub enum InitWorkspaceError {
    #[error("Workspace already exists at path: {0}")]
    WorkspaceAlreadyExists(String),
    
    #[error("Git clone failed: {0}")]
    GitCloneFailed(String),
    
    #[error("Failed to create workspace config: {0}")]
    ConfigCreationFailed(String),
    
    #[error("Invalid manifest URL: {0}")]
    InvalidManifestUrl(String),
    
    #[error("Failed to read manifest: {0}")]
    ManifestReadFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Git URL error: {0}")]
    GitUrlError(#[from] crate::domain::value_objects::git_url::GitUrlError),
    
    #[error("File path error: {0}")]
    FilePathError(#[from] crate::domain::value_objects::file_path::FilePathError),
}

/// ワークスペース初期化の設定
#[derive(Debug, Clone)]
pub struct InitWorkspaceConfig {
    /// マニフェストのURL
    pub manifest_url: GitUrl,
    
    /// ワークスペースのルートパス
    pub workspace_path: FilePath,
    
    /// 使用するブランチ（指定しない場合はデフォルトブランチ）
    pub branch: Option<String>,
    
    /// フィルタリングするグループ（指定しない場合は全て）
    pub groups: Option<Vec<String>>,
    
    /// シャローコピーを使用するか
    pub shallow: bool,
    
    /// 既存のワークスペースを強制上書きするか
    pub force: bool,
}

/// ワークスペース初期化のユースケース
pub struct InitWorkspaceUseCase {
    /// 作業用の設定
    config: InitWorkspaceConfig,
}

impl InitWorkspaceUseCase {
    /// 新しいInitWorkspaceUseCaseインスタンスを作成
    pub fn new(config: InitWorkspaceConfig) -> Self {
        Self { config }
    }

    /// ワークスペースを初期化
    pub async fn execute(&self) -> Result<Workspace, InitWorkspaceError> {
        // 1. ワークスペースパスの存在チェック
        self.check_workspace_path()?;
        
        // 2. マニフェストリポジトリをクローン
        let manifest_path = self.clone_manifest_repository().await?;
        
        // 3. マニフェストファイルの読み込み
        let manifest = self.read_manifest(&manifest_path).await?;
        
        // 4. .tsrc/config.yml の作成
        self.create_workspace_config(&manifest).await?;
        
        // 5. ワークスペースエンティティを作成
        let workspace = self.create_workspace(manifest)?;
        
        Ok(workspace)
    }
    
    /// ワークスペースパスの存在チェック
    fn check_workspace_path(&self) -> Result<(), InitWorkspaceError> {
        let workspace_path = self.config.workspace_path.to_path_buf();
        
        if workspace_path.exists() {
            if !self.config.force {
                return Err(InitWorkspaceError::WorkspaceAlreadyExists(
                    workspace_path.display().to_string()
                ));
            }
            
            // 強制フラグが設定されている場合は既存のディレクトリを削除
            if workspace_path.is_dir() {
                std::fs::remove_dir_all(&workspace_path)?;
            } else {
                std::fs::remove_file(&workspace_path)?;
            }
        }
        
        // ディレクトリを作成
        std::fs::create_dir_all(&workspace_path)?;
        
        Ok(())
    }
    
    /// マニフェストリポジトリをクローン
    async fn clone_manifest_repository(&self) -> Result<PathBuf, InitWorkspaceError> {
        let workspace_path = self.config.workspace_path.to_path_buf();
        let manifest_dir = workspace_path.join(".tsrc").join("manifest");
        
        // .tsrcディレクトリを作成
        let tsrc_dir = workspace_path.join(".tsrc");
        std::fs::create_dir_all(&tsrc_dir)?;
        
        // Git clone実行（実際のGit操作は別途実装予定）
        // ここでは疑似的な実装
        self.perform_git_clone(&self.config.manifest_url, &manifest_dir).await?;
        
        Ok(manifest_dir)
    }
    
    /// Git clone実行（疑似実装）
    async fn perform_git_clone(&self, url: &GitUrl, target_path: &PathBuf) -> Result<(), InitWorkspaceError> {
        // TODO: 実際のGit操作はインフラストラクチャ層で実装
        // ここでは疑似的にディレクトリとマニフェストファイルを作成
        std::fs::create_dir_all(target_path)?;
        
        // 仮のmanifest.ymlファイルを作成（本来はGitからクローン）
        let manifest_file = target_path.join("manifest.yml");
        let default_manifest = format!(
            r#"repos:
  - dest: example-repo
    url: {}
    branch: main
"#,
            url.as_str()
        );
        std::fs::write(manifest_file, default_manifest)?;
        
        Ok(())
    }
    
    /// マニフェストファイルの読み込み
    async fn read_manifest(&self, manifest_path: &PathBuf) -> Result<Manifest, InitWorkspaceError> {
        let manifest_file = manifest_path.join("manifest.yml");
        
        if !manifest_file.exists() {
            return Err(InitWorkspaceError::ManifestReadFailed(
                "manifest.yml not found".to_string()
            ));
        }
        
        let content = std::fs::read_to_string(&manifest_file)?;
        
        // YAMLパースは別途実装予定
        // ここでは簡易的な実装
        let manifest = self.parse_manifest_yaml(&content)?;
        
        Ok(manifest)
    }
    
    /// YAML文字列をManifestエンティティにパース（簡易実装）
    fn parse_manifest_yaml(&self, _content: &str) -> Result<Manifest, InitWorkspaceError> {
        // TODO: 実際のYAMLパース処理を実装
        // ここでは仮のManifestを返す
        Ok(Manifest::new(vec![]))
    }
    
    /// .tsrc/config.yml の作成
    async fn create_workspace_config(&self, manifest: &Manifest) -> Result<(), InitWorkspaceError> {
        let workspace_path = self.config.workspace_path.to_path_buf();
        let config_dir = workspace_path.join(".tsrc");
        let config_file = config_dir.join("config.yml");
        
        // 設定ファイルの内容を作成
        let config_content = self.generate_config_yaml(manifest)?;
        
        // ファイルに書き込み
        std::fs::write(&config_file, config_content)?;
        
        Ok(())
    }
    
    /// config.yamlの内容を生成
    fn generate_config_yaml(&self, _manifest: &Manifest) -> Result<String, InitWorkspaceError> {
        let config = WorkspaceConfigFile {
            manifest_url: self.config.manifest_url.as_str().to_string(),
            manifest_branch: self.config.branch.clone(),
            groups: self.config.groups.clone(),
            shallow: self.config.shallow,
        };
        
        // TODO: 実際のYAMLシリアライズ処理を実装
        // ここでは簡易的な文字列生成
        let yaml_content = format!(
            r#"manifest_url: {}
manifest_branch: {}
groups: {}
shallow: {}
"#,
            config.manifest_url,
            config.manifest_branch.unwrap_or_else(|| "main".to_string()),
            config.groups.map(|g| format!("{:?}", g)).unwrap_or_else(|| "[]".to_string()),
            config.shallow
        );
        
        Ok(yaml_content)
    }
    
    /// ワークスペースエンティティを作成
    fn create_workspace(&self, manifest: Manifest) -> Result<Workspace, InitWorkspaceError> {
        let workspace_config = WorkspaceConfig::new(
            self.config.manifest_url.as_str(),
            self.config.branch.as_deref().unwrap_or("main")
        );
        
        let workspace = Workspace::new(
            self.config.workspace_path.to_path_buf(),
            workspace_config,
        ).with_manifest(manifest);
        
        Ok(workspace)
    }
}

/// ワークスペース設定ファイルの構造
#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceConfigFile {
    /// マニフェストURL
    manifest_url: String,
    
    /// マニフェストブランチ
    manifest_branch: Option<String>,
    
    /// 使用するグループ
    groups: Option<Vec<String>>,
    
    /// シャローコピー設定
    shallow: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_init_workspace_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = FilePath::new_absolute(temp_dir.path().to_str().unwrap()).unwrap();
        let manifest_url = GitUrl::new("https://github.com/example/manifest.git").unwrap();
        
        let config = InitWorkspaceConfig {
            manifest_url,
            workspace_path,
            branch: Some("main".to_string()),
            groups: Some(vec!["default".to_string()]),
            shallow: false,
            force: false,
        };
        
        let use_case = InitWorkspaceUseCase::new(config);
        assert_eq!(use_case.config.branch, Some("main".to_string()));
    }
    
    #[test]
    fn test_workspace_path_validation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = FilePath::new_absolute(temp_dir.path().to_str().unwrap()).unwrap();
        let manifest_url = GitUrl::new("https://github.com/example/manifest.git").unwrap();
        
        let config = InitWorkspaceConfig {
            manifest_url,
            workspace_path,
            branch: None,
            groups: None,
            shallow: false,
            force: true, // TempDirは既に存在するのでforceフラグを設定
        };
        
        let use_case = InitWorkspaceUseCase::new(config);
        
        // 空のディレクトリなので成功するはず
        assert!(use_case.check_workspace_path().is_ok());
    }
    
    #[tokio::test]
    async fn test_generate_config_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = FilePath::new_absolute(temp_dir.path().to_str().unwrap()).unwrap();
        let manifest_url = GitUrl::new("https://github.com/example/manifest.git").unwrap();
        
        let config = InitWorkspaceConfig {
            manifest_url,
            workspace_path,
            branch: Some("develop".to_string()),
            groups: Some(vec!["group1".to_string(), "group2".to_string()]),
            shallow: true,
            force: false,
        };
        
        let use_case = InitWorkspaceUseCase::new(config);
        let manifest = Manifest::new(vec![]);
        
        let yaml_content = use_case.generate_config_yaml(&manifest).unwrap();
        
        assert!(yaml_content.contains("manifest_url: https://github.com/example/manifest"));
        assert!(yaml_content.contains("manifest_branch: develop"));
        assert!(yaml_content.contains("shallow: true"));
    }
}