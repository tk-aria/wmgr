use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// BranchName関連のエラー
#[derive(Debug, Error, PartialEq)]
pub enum BranchNameError {
    #[error("Branch name cannot be empty")]
    Empty,
    
    #[error("Branch name too long: {0} characters (max: 255)")]
    TooLong(usize),
    
    #[error("Invalid character in branch name: {0}")]
    InvalidCharacter(String),
    
    #[error("Branch name cannot start with '-': {0}")]
    StartsWithHyphen(String),
    
    #[error("Branch name cannot end with '.lock': {0}")]
    EndsWithLock(String),
    
    #[error("Branch name contains consecutive dots: {0}")]
    ConsecutiveDots(String),
    
    #[error("Reserved branch name: {0}")]
    Reserved(String),
}

/// Gitブランチ名の値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchName {
    /// 検証済みブランチ名
    name: String,
}

impl BranchName {
    /// 新しいBranchNameインスタンスを作成
    pub fn new(name: &str) -> Result<Self, BranchNameError> {
        Self::validate(name)?;
        Ok(Self {
            name: name.to_string(),
        })
    }
    
    /// ブランチ名の妥当性を検証
    fn validate(name: &str) -> Result<(), BranchNameError> {
        // 空文字チェック
        if name.is_empty() {
            return Err(BranchNameError::Empty);
        }
        
        // 長さチェック（255文字まで）
        if name.len() > 255 {
            return Err(BranchNameError::TooLong(name.len()));
        }
        
        // ハイフンで始まるかチェック
        if name.starts_with('-') {
            return Err(BranchNameError::StartsWithHyphen(name.to_string()));
        }
        
        // .lockで終わるかチェック
        if name.ends_with(".lock") {
            return Err(BranchNameError::EndsWithLock(name.to_string()));
        }
        
        // 予約語チェック
        if matches!(name, "HEAD" | "ORIG_HEAD" | "FETCH_HEAD" | "MERGE_HEAD") {
            return Err(BranchNameError::Reserved(name.to_string()));
        }
        
        // 不正な文字をチェック
        // ASCII制御文字、スペース、~、^、:、?、*、[、\、DEL
        for ch in name.chars() {
            if ch.is_ascii_control() 
                || ch == ' ' 
                || ch == '~' 
                || ch == '^' 
                || ch == ':' 
                || ch == '?' 
                || ch == '*' 
                || ch == '[' 
                || ch == '\\' 
                || ch == '\x7F' 
            {
                return Err(BranchNameError::InvalidCharacter(ch.to_string()));
            }
        }
        
        // 連続するドットをチェック
        if name.contains("..") {
            return Err(BranchNameError::ConsecutiveDots(name.to_string()));
        }
        
        // リモート追跡ブランチの形式チェック（refs/heads/やrefs/remotes/で始まらない）
        // ただし、これらは完全に無効ではなく、特別な処理が必要な場合がある
        
        Ok(())
    }
    
    /// ブランチ名を文字列として取得
    pub fn as_str(&self) -> &str {
        &self.name
    }
    
    /// ブランチ名を所有権付きで取得
    pub fn into_string(self) -> String {
        self.name
    }
    
    /// デフォルトブランチかどうかを判定
    pub fn is_default_branch(&self) -> bool {
        matches!(self.name.as_str(), "main" | "master" | "develop" | "development")
    }
    
    /// リリースブランチかどうかを判定
    pub fn is_release_branch(&self) -> bool {
        let release_patterns = [
            r"^release/.*",
            r"^releases/.*", 
            r"^rel/.*",
            r"^v\d+\.\d+.*",
        ];
        
        release_patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(&self.name)
        })
    }
    
    /// フィーチャーブランチかどうかを判定
    pub fn is_feature_branch(&self) -> bool {
        let feature_patterns = [
            r"^feature/.*",
            r"^feat/.*",
            r"^features/.*",
        ];
        
        feature_patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(&self.name)
        })
    }
    
    /// ホットフィックスブランチかどうかを判定
    pub fn is_hotfix_branch(&self) -> bool {
        let hotfix_patterns = [
            r"^hotfix/.*",
            r"^hotfixes/.*",
            r"^fix/.*",
        ];
        
        hotfix_patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(&self.name)
        })
    }
    
    /// ブランチの種類を取得
    pub fn branch_type(&self) -> BranchType {
        if self.is_default_branch() {
            BranchType::Default
        } else if self.is_release_branch() {
            BranchType::Release
        } else if self.is_feature_branch() {
            BranchType::Feature
        } else if self.is_hotfix_branch() {
            BranchType::Hotfix
        } else {
            BranchType::Other
        }
    }
}

/// ブランチの種類
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchType {
    Default,
    Release,
    Feature,
    Hotfix,
    Other,
}

impl fmt::Display for BranchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<&str> for BranchName {
    type Error = BranchNameError;
    
    fn try_from(name: &str) -> Result<Self, Self::Error> {
        BranchName::new(name)
    }
}

impl TryFrom<String> for BranchName {
    type Error = BranchNameError;
    
    fn try_from(name: String) -> Result<Self, Self::Error> {
        BranchName::new(&name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_branch_names() {
        let valid_names = [
            "main",
            "master", 
            "develop",
            "feature/user-auth",
            "release/1.0.0",
            "hotfix/critical-bug",
            "my-branch",
            "branch_name",
            "branch.name",
            "123",
        ];
        
        for name in valid_names {
            assert!(BranchName::new(name).is_ok(), "Failed for: {}", name);
        }
    }
    
    #[test]
    fn test_invalid_branch_names() {
        let invalid_cases = [
            ("", BranchNameError::Empty),
            ("-branch", BranchNameError::StartsWithHyphen("-branch".to_string())),
            ("branch.lock", BranchNameError::EndsWithLock("branch.lock".to_string())),
            ("branch..name", BranchNameError::ConsecutiveDots("branch..name".to_string())),
            ("HEAD", BranchNameError::Reserved("HEAD".to_string())),
            ("branch name", BranchNameError::InvalidCharacter(" ".to_string())),
            ("branch:name", BranchNameError::InvalidCharacter(":".to_string())),
        ];
        
        for (name, expected_error) in invalid_cases {
            let result = BranchName::new(name);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), expected_error);
        }
    }
    
    #[test]
    fn test_branch_type_detection() {
        let test_cases = [
            ("main", BranchType::Default),
            ("master", BranchType::Default),
            ("feature/user-auth", BranchType::Feature),
            ("feat/new-ui", BranchType::Feature),
            ("release/1.0.0", BranchType::Release),
            ("rel/v2.1", BranchType::Release),
            ("hotfix/critical-bug", BranchType::Hotfix),
            ("fix/memory-leak", BranchType::Hotfix),
            ("random-branch", BranchType::Other),
        ];
        
        for (name, expected_type) in test_cases {
            let branch = BranchName::new(name).unwrap();
            assert_eq!(branch.branch_type(), expected_type, "Failed for: {}", name);
        }
    }
    
    #[test]
    fn test_is_default_branch() {
        let default_branches = ["main", "master", "develop"];
        let non_default_branches = ["feature/test", "release/1.0", "other"];
        
        for name in default_branches {
            let branch = BranchName::new(name).unwrap();
            assert!(branch.is_default_branch(), "Failed for: {}", name);
        }
        
        for name in non_default_branches {
            let branch = BranchName::new(name).unwrap();
            assert!(!branch.is_default_branch(), "Failed for: {}", name);
        }
    }
    
    #[test]
    fn test_branch_name_too_long() {
        let long_name = "a".repeat(256);
        let result = BranchName::new(&long_name);
        assert!(matches!(result, Err(BranchNameError::TooLong(256))));
    }
}