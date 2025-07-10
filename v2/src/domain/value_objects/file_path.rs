use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf, Component};
use std::fmt;
use thiserror::Error;

/// FilePath関連のエラー
#[derive(Debug, Error, PartialEq)]
pub enum FilePathError {
    #[error("Path cannot be empty")]
    Empty,
    
    #[error("Path contains null bytes")]
    ContainsNullBytes,
    
    #[error("Path contains path traversal: {0}")]
    PathTraversal(String),
    
    #[error("Path is absolute but relative path expected: {0}")]
    UnexpectedAbsolute(String),
    
    #[error("Path contains invalid characters: {0}")]
    InvalidCharacters(String),
    
    #[error("Path too long: {0} characters")]
    TooLong(usize),
}

/// ファイルパスの値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FilePath {
    /// 正規化されたパス文字列
    path: String,
    
    /// パスが絶対パスかどうか
    is_absolute: bool,
}

impl FilePath {
    /// 新しいFilePathインスタンスを作成（相対パス専用）
    pub fn new_relative(path: &str) -> Result<Self, FilePathError> {
        Self::validate_and_create(path, false)
    }
    
    /// 新しいFilePathインスタンスを作成（絶対パス専用）
    pub fn new_absolute(path: &str) -> Result<Self, FilePathError> {
        Self::validate_and_create(path, true)
    }
    
    /// 新しいFilePathインスタンスを作成（自動判定）
    pub fn new(path: &str) -> Result<Self, FilePathError> {
        let path_buf = PathBuf::from(path);
        let is_absolute = path_buf.is_absolute();
        Self::validate_and_create(path, is_absolute)
    }
    
    /// パスの検証と作成
    fn validate_and_create(path: &str, expected_absolute: bool) -> Result<Self, FilePathError> {
        // 空文字チェック
        if path.is_empty() {
            return Err(FilePathError::Empty);
        }
        
        // NULLバイトチェック
        if path.contains('\0') {
            return Err(FilePathError::ContainsNullBytes);
        }
        
        // 長さチェック（4096文字まで）
        if path.len() > 4096 {
            return Err(FilePathError::TooLong(path.len()));
        }
        
        let path_buf = PathBuf::from(path);
        let is_absolute = path_buf.is_absolute();
        
        // 絶対パス期待だが相対パスまたはその逆
        if expected_absolute && !is_absolute {
            return Err(FilePathError::UnexpectedAbsolute(path.to_string()));
        }
        
        // パストラバーサルチェック
        Self::check_path_traversal(&path_buf)?;
        
        // プラットフォーム固有の不正文字チェック
        Self::check_invalid_characters(path)?;
        
        // パスを正規化
        let normalized = Self::normalize_path(&path_buf);
        
        Ok(Self {
            path: normalized,
            is_absolute,
        })
    }
    
    /// パストラバーサル攻撃のチェック
    fn check_path_traversal(path: &Path) -> Result<(), FilePathError> {
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    return Err(FilePathError::PathTraversal(
                        path.display().to_string()
                    ));
                }
                Component::CurDir => {
                    // ./ は通常問題ないが、複数連続する場合は問題となる可能性
                    continue;
                }
                _ => continue,
            }
        }
        Ok(())
    }
    
    /// プラットフォーム固有の不正文字チェック
    fn check_invalid_characters(_path: &str) -> Result<(), FilePathError> {
        // Windows固有の不正文字
        #[cfg(target_os = "windows")]
        {
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            for ch in _path.chars() {
                if invalid_chars.contains(&ch) || (ch as u32) < 32 {
                    return Err(FilePathError::InvalidCharacters(ch.to_string()));
                }
            }
            
            // Windows予約名チェック
            let reserved_names = [
                "CON", "PRN", "AUX", "NUL",
                "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
                "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
            ];
            
            for component in Path::new(_path).components() {
                if let Component::Normal(os_str) = component {
                    if let Some(name) = os_str.to_str() {
                        let name_upper = name.to_uppercase();
                        if reserved_names.contains(&name_upper.as_str()) {
                            return Err(FilePathError::InvalidCharacters(name.to_string()));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// パスの正規化
    fn normalize_path(path: &Path) -> String {
        // パスの正規化（余分なスラッシュの除去など）
        let mut normalized = PathBuf::new();
        
        for component in path.components() {
            match component {
                Component::RootDir => normalized.push("/"),
                Component::Normal(part) => normalized.push(part),
                Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
                // CurDir (./) と ParentDir (../) は既にチェック済み
                _ => {}
            }
        }
        
        normalized.display().to_string()
    }
    
    /// パス文字列を取得
    pub fn as_str(&self) -> &str {
        &self.path
    }
    
    /// パスを所有権付きで取得
    pub fn into_string(self) -> String {
        self.path
    }
    
    /// PathBufとして取得
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
    
    /// Pathとして取得
    pub fn as_path(&self) -> &Path {
        Path::new(&self.path)
    }
    
    /// 絶対パスかどうか
    pub fn is_absolute(&self) -> bool {
        self.is_absolute
    }
    
    /// 相対パスかどうか
    pub fn is_relative(&self) -> bool {
        !self.is_absolute
    }
    
    /// ファイル名を取得
    pub fn file_name(&self) -> Option<&str> {
        self.as_path().file_name()?.to_str()
    }
    
    /// ファイル拡張子を取得
    pub fn extension(&self) -> Option<&str> {
        self.as_path().extension()?.to_str()
    }
    
    /// 親ディレクトリを取得
    pub fn parent(&self) -> Option<FilePath> {
        let parent_path = self.as_path().parent()?;
        if parent_path.as_os_str().is_empty() {
            return None;
        }
        
        FilePath::new(&parent_path.display().to_string()).ok()
    }
    
    /// パスを結合
    pub fn join(&self, path: &str) -> Result<FilePath, FilePathError> {
        let joined = self.to_path_buf().join(path);
        FilePath::new(&joined.display().to_string())
    }
    
    /// ワークスペースルートからの相対パスに変換
    pub fn strip_workspace_prefix(&self, workspace_root: &FilePath) -> Option<FilePath> {
        if !self.is_absolute || !workspace_root.is_absolute {
            return None;
        }
        
        let self_path = self.as_path();
        let root_path = workspace_root.as_path();
        
        if let Ok(relative) = self_path.strip_prefix(root_path) {
            FilePath::new_relative(&relative.display().to_string()).ok()
        } else {
            None
        }
    }
    
    /// 安全なパスかどうかチェック（サニタイズ済み）
    pub fn is_safe(&self) -> bool {
        // 既に検証済みなので常にtrue
        true
    }
}

impl fmt::Display for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl TryFrom<&str> for FilePath {
    type Error = FilePathError;
    
    fn try_from(path: &str) -> Result<Self, Self::Error> {
        FilePath::new(path)
    }
}

impl TryFrom<String> for FilePath {
    type Error = FilePathError;
    
    fn try_from(path: String) -> Result<Self, Self::Error> {
        FilePath::new(&path)
    }
}

impl TryFrom<&Path> for FilePath {
    type Error = FilePathError;
    
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        FilePath::new(&path.display().to_string())
    }
}

impl TryFrom<PathBuf> for FilePath {
    type Error = FilePathError;
    
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        FilePath::new(&path.display().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_relative_paths() {
        let valid_paths = [
            "file.txt",
            "dir/file.txt",
            "dir/subdir/file.txt",
            "dir_name",
            "file-name.txt",
            "file_name.txt",
        ];
        
        for path in valid_paths {
            assert!(FilePath::new_relative(path).is_ok(), "Failed for: {}", path);
        }
    }
    
    #[test]
    fn test_valid_absolute_paths() {
        #[cfg(unix)]
        let valid_paths = [
            "/home/user/file.txt",
            "/usr/local/bin/app",
            "/var/log/app.log",
        ];
        
        #[cfg(windows)]
        let valid_paths = [
            "C:\\Users\\user\\file.txt",
            "D:\\Program Files\\app\\app.exe",
        ];
        
        for path in valid_paths {
            assert!(FilePath::new_absolute(path).is_ok(), "Failed for: {}", path);
        }
    }
    
    #[test]
    fn test_invalid_paths() {
        let invalid_cases = [
            ("", FilePathError::Empty),
            ("../../../etc/passwd", FilePathError::PathTraversal("../../../etc/passwd".to_string())),
            ("dir/../../../file", FilePathError::PathTraversal("dir/../../../file".to_string())),
        ];
        
        for (path, expected_error) in invalid_cases {
            let result = FilePath::new(path);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), expected_error);
        }
    }
    
    #[test]
    fn test_path_operations() {
        let file_path = FilePath::new_relative("dir/subdir/file.txt").unwrap();
        
        assert_eq!(file_path.file_name(), Some("file.txt"));
        assert_eq!(file_path.extension(), Some("txt"));
        assert!(file_path.is_relative());
        assert!(!file_path.is_absolute());
        
        let parent = file_path.parent().unwrap();
        assert_eq!(parent.as_str(), "dir/subdir");
    }
    
    #[test]
    fn test_path_join() {
        let base = FilePath::new_relative("base/dir").unwrap();
        let joined = base.join("file.txt").unwrap();
        
        // プラットフォーム依存の結果になるため、存在チェックのみ
        assert!(joined.as_str().contains("file.txt"));
    }
    
    #[test]
    fn test_null_byte_rejection() {
        let path_with_null = "file\0.txt";
        let result = FilePath::new(path_with_null);
        assert!(matches!(result, Err(FilePathError::ContainsNullBytes)));
    }
    
    #[test]
    fn test_too_long_path() {
        let long_path = "a".repeat(4097);
        let result = FilePath::new(&long_path);
        assert!(matches!(result, Err(FilePathError::TooLong(4097))));
    }
    
    #[cfg(windows)]
    #[test]
    fn test_windows_reserved_names() {
        let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];
        
        for name in reserved_names {
            let result = FilePath::new(name);
            assert!(result.is_err(), "Should reject reserved name: {}", name);
        }
    }
}