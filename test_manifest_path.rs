use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tsrc::domain::entities::workspace::Workspace;

fn main() {
    // テスト用一時ディレクトリを作成
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path().to_path_buf();
    
    println!("Testing manifest path resolution...");
    println!("Workspace root: {}", workspace_root.display());
    
    // Workspaceオブジェクトを作成
    let workspace = Workspace::new(workspace_root.clone());
    
    // 1. manifest.yml も .wmgr/manifest.yml も存在しない場合
    println!("\n1. No manifest files exist:");
    let manifest_path = workspace.manifest_file_path();
    println!("Expected: {}/manifest.yml", workspace_root.display());
    println!("Actual:   {}", manifest_path.display());
    assert_eq!(manifest_path, workspace_root.join("manifest.yml"));
    
    // 2. .wmgr/manifest.yml のみ存在する場合
    println!("\n2. Only .wmgr/manifest.yml exists:");
    let wmgr_dir = workspace_root.join(".wmgr");
    fs::create_dir_all(&wmgr_dir).unwrap();
    fs::write(wmgr_dir.join("manifest.yml"), "# test manifest").unwrap();
    
    let manifest_path = workspace.manifest_file_path();
    println!("Expected: {}/.wmgr/manifest.yml", workspace_root.display());
    println!("Actual:   {}", manifest_path.display());
    assert_eq!(manifest_path, workspace_root.join(".wmgr").join("manifest.yml"));
    
    // 3. 両方存在する場合（カレントディレクトリが優先される）
    println!("\n3. Both manifest.yml and .wmgr/manifest.yml exist:");
    fs::write(workspace_root.join("manifest.yml"), "# current manifest").unwrap();
    
    let manifest_path = workspace.manifest_file_path();
    println!("Expected: {}/manifest.yml", workspace_root.display());
    println!("Actual:   {}", manifest_path.display());
    assert_eq!(manifest_path, workspace_root.join("manifest.yml"));
    
    // 4. レガシーパスのテスト
    println!("\n4. Legacy path test:");
    let legacy_path = workspace.legacy_manifest_file_path();
    println!("Expected: {}/.tsrc/manifest.yml", workspace_root.display());
    println!("Actual:   {}", legacy_path.display());
    assert_eq!(legacy_path, workspace_root.join(".tsrc").join("manifest.yml"));
    
    println!("\n✅ All tests passed!");
}