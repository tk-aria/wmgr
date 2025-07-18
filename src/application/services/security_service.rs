use std::process::Command;
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// セキュリティ関連のエラー
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Dependency audit failed: {0}")]
    AuditFailed(String),
    
    #[error("Command execution failed: {0}")]
    CommandExecutionFailed(String),
    
    #[error("JSON parsing failed: {0}")]
    JsonParsingFailed(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}

/// 脆弱性の重要度レベル
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VulnerabilitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 脆弱性情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// 脆弱性ID
    pub id: String,
    
    /// パッケージ名
    pub package: String,
    
    /// 影響を受けるバージョン
    pub version: String,
    
    /// 脆弱性の重要度
    pub severity: VulnerabilitySeverity,
    
    /// 脆弱性の説明
    pub description: String,
    
    /// 修正版が利用可能か
    pub patched_versions: Vec<String>,
    
    /// 脆弱性のURL
    pub url: Option<String>,
}

/// 監査結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    /// 発見された脆弱性のリスト
    pub vulnerabilities: Vec<Vulnerability>,
    
    /// 監査実行時刻
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// 監査されたプロジェクトパス
    pub project_path: String,
    
    /// 警告の数（重要度別）
    pub warning_count: AuditSummary,
}

/// 監査結果の概要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

impl AuditSummary {
    pub fn new() -> Self {
        Self {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
        }
    }
    
    pub fn total(&self) -> usize {
        self.critical + self.high + self.medium + self.low
    }
    
    pub fn has_critical_or_high(&self) -> bool {
        self.critical > 0 || self.high > 0
    }
}

/// セキュリティサービス
pub struct SecurityService {
    /// cargo-auditのパス（デフォルトは"cargo"）
    cargo_path: String,
}

impl SecurityService {
    /// 新しいSecurityServiceインスタンスを作成
    pub fn new() -> Self {
        Self {
            cargo_path: "cargo".to_string(),
        }
    }
    
    /// カスタムのcargoパスを指定
    pub fn with_cargo_path(mut self, cargo_path: impl Into<String>) -> Self {
        self.cargo_path = cargo_path.into();
        self
    }
    
    /// 依存関係の脆弱性監査を実行
    pub async fn audit_dependencies(&self, project_path: &Path) -> Result<AuditResult, SecurityError> {
        // cargo-auditが利用可能かチェック
        self.check_cargo_audit_available()?;
        
        // 監査実行
        let output = self.run_cargo_audit(project_path).await?;
        
        // 結果をパース
        let vulnerabilities = self.parse_audit_output(&output)?;
        
        // 概要を計算
        let warning_count = self.calculate_summary(&vulnerabilities);
        
        Ok(AuditResult {
            vulnerabilities,
            timestamp: chrono::Utc::now(),
            project_path: project_path.display().to_string(),
            warning_count,
        })
    }
    
    /// 依存関係の脆弱性監査を実行（サイレントモード）
    pub async fn audit_dependencies_silent(&self, project_path: &Path) -> Result<bool, SecurityError> {
        let result = self.audit_dependencies(project_path).await?;
        Ok(!result.warning_count.has_critical_or_high())
    }
    
    /// cargo-auditが利用可能かチェック
    fn check_cargo_audit_available(&self) -> Result<(), SecurityError> {
        let output = Command::new(&self.cargo_path)
            .args(&["audit", "--version"])
            .output()
            .map_err(|e| SecurityError::CommandExecutionFailed(
                format!("cargo-audit is not available: {}", e)
            ))?;
        
        if !output.status.success() {
            return Err(SecurityError::CommandExecutionFailed(
                "cargo-audit is not installed. Run 'cargo install cargo-audit' to install.".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// cargo-auditを実行
    async fn run_cargo_audit(&self, project_path: &Path) -> Result<String, SecurityError> {
        let output = Command::new(&self.cargo_path)
            .args(&["audit", "--json"])
            .current_dir(project_path)
            .output()
            .map_err(|e| SecurityError::CommandExecutionFailed(
                format!("Failed to execute cargo audit: {}", e)
            ))?;
        
        let stdout = std::str::from_utf8(&output.stdout)?;
        let stderr = std::str::from_utf8(&output.stderr)?;
        
        // cargo-auditは脆弱性が見つかった場合に非ゼロの終了コードを返すが、
        // これは正常な動作なので、stderrが空でない場合のみエラーとする
        if !output.status.success() && !stderr.is_empty() {
            return Err(SecurityError::AuditFailed(stderr.to_string()));
        }
        
        Ok(stdout.to_string())
    }
    
    /// 監査結果をパース
    fn parse_audit_output(&self, output: &str) -> Result<Vec<Vulnerability>, SecurityError> {
        if output.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        // cargo-auditのJSON出力をパース
        let mut vulnerabilities = Vec::new();
        
        // cargo-auditの出力は行ごとにJSONオブジェクトが含まれている場合がある
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            // JSONをパースして脆弱性情報を抽出
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(json) => {
                    if let Some(vuln) = self.extract_vulnerability_from_json(&json)? {
                        vulnerabilities.push(vuln);
                    }
                }
                Err(_) => {
                    // JSON形式でない行はスキップ（警告メッセージなど）
                    continue;
                }
            }
        }
        
        Ok(vulnerabilities)
    }
    
    /// JSONから脆弱性情報を抽出
    fn extract_vulnerability_from_json(&self, json: &serde_json::Value) -> Result<Option<Vulnerability>, SecurityError> {
        // cargo-auditのJSON形式に従って脆弱性情報を抽出
        if let Some(vulnerabilities) = json.get("vulnerabilities") {
            if let Some(vuln_array) = vulnerabilities.as_array() {
                for vuln_obj in vuln_array {
                    if let Some(advisory) = vuln_obj.get("advisory") {
                        let vulnerability = Vulnerability {
                            id: advisory.get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            package: vuln_obj.get("package")
                                .and_then(|p| p.get("name"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            version: vuln_obj.get("package")
                                .and_then(|p| p.get("version"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            severity: self.parse_severity(
                                advisory.get("cvss")
                                    .and_then(|c| c.get("severity"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("medium")
                            ),
                            description: advisory.get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("No description available")
                                .to_string(),
                            patched_versions: advisory.get("patched_versions")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect())
                                .unwrap_or_default(),
                            url: advisory.get("url")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                        };
                        return Ok(Some(vulnerability));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// 重要度文字列をパース
    fn parse_severity(&self, severity_str: &str) -> VulnerabilitySeverity {
        match severity_str.to_lowercase().as_str() {
            "low" => VulnerabilitySeverity::Low,
            "medium" => VulnerabilitySeverity::Medium,
            "high" => VulnerabilitySeverity::High,
            "critical" => VulnerabilitySeverity::Critical,
            _ => VulnerabilitySeverity::Medium,
        }
    }
    
    /// 概要を計算
    fn calculate_summary(&self, vulnerabilities: &[Vulnerability]) -> AuditSummary {
        let mut summary = AuditSummary::new();
        
        for vuln in vulnerabilities {
            match vuln.severity {
                VulnerabilitySeverity::Critical => summary.critical += 1,
                VulnerabilitySeverity::High => summary.high += 1,
                VulnerabilitySeverity::Medium => summary.medium += 1,
                VulnerabilitySeverity::Low => summary.low += 1,
            }
        }
        
        summary
    }
}

impl Default for SecurityService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[tokio::test]
    async fn test_security_service_creation() {
        let service = SecurityService::new();
        assert_eq!(service.cargo_path, "cargo");
        
        let service = SecurityService::new().with_cargo_path("custom-cargo");
        assert_eq!(service.cargo_path, "custom-cargo");
    }
    
    #[test]
    fn test_vulnerability_severity_parsing() {
        let service = SecurityService::new();
        
        assert_eq!(service.parse_severity("low"), VulnerabilitySeverity::Low);
        assert_eq!(service.parse_severity("medium"), VulnerabilitySeverity::Medium);
        assert_eq!(service.parse_severity("high"), VulnerabilitySeverity::High);
        assert_eq!(service.parse_severity("critical"), VulnerabilitySeverity::Critical);
        assert_eq!(service.parse_severity("unknown"), VulnerabilitySeverity::Medium);
    }
    
    #[test]
    fn test_audit_summary() {
        let mut summary = AuditSummary::new();
        assert_eq!(summary.total(), 0);
        assert!(!summary.has_critical_or_high());
        
        summary.high = 1;
        summary.medium = 2;
        assert_eq!(summary.total(), 3);
        assert!(summary.has_critical_or_high());
        
        summary.critical = 1;
        assert_eq!(summary.total(), 4);
        assert!(summary.has_critical_or_high());
    }
    
    #[test]
    fn test_parse_audit_output_empty() {
        let service = SecurityService::new();
        let result = service.parse_audit_output("").unwrap();
        assert!(result.is_empty());
    }
    
    #[test]
    fn test_calculate_summary() {
        let service = SecurityService::new();
        let vulnerabilities = vec![
            Vulnerability {
                id: "1".to_string(),
                package: "test".to_string(),
                version: "1.0.0".to_string(),
                severity: VulnerabilitySeverity::Critical,
                description: "Test".to_string(),
                patched_versions: vec![],
                url: None,
            },
            Vulnerability {
                id: "2".to_string(),
                package: "test".to_string(),
                version: "1.0.0".to_string(),
                severity: VulnerabilitySeverity::High,
                description: "Test".to_string(),
                patched_versions: vec![],
                url: None,
            },
            Vulnerability {
                id: "3".to_string(),
                package: "test".to_string(),
                version: "1.0.0".to_string(),
                severity: VulnerabilitySeverity::Medium,
                description: "Test".to_string(),
                patched_versions: vec![],
                url: None,
            },
        ];
        
        let summary = service.calculate_summary(&vulnerabilities);
        assert_eq!(summary.critical, 1);
        assert_eq!(summary.high, 1);
        assert_eq!(summary.medium, 1);
        assert_eq!(summary.low, 0);
        assert_eq!(summary.total(), 3);
        assert!(summary.has_critical_or_high());
    }
}