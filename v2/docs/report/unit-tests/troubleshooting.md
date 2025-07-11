# 単体テスト実装時のトラブルシューティング

## テスト失敗の修正

### 失敗1: application::use_cases::status_check::tests::test_repository_status_creation
**問題:** `assertion failed: !status.has_issues()`
**原因:** テストでRepositoryStatusが想定と異なる状態になっている

### 失敗2: application::use_cases::init_workspace::tests::test_workspace_path_validation
**問題:** `assertion failed: use_case.check_workspace_path().is_ok()`
**原因:** ワークスペースパスの検証ロジックに問題

### 失敗3: application::use_cases::init_workspace::tests::test_generate_config_yaml
**問題:** `assertion failed: yaml_content.contains("manifest_url: https://github.com/example/manifest.git")`
**原因:** 生成されるYAMLの形式が期待値と異なる

### 失敗4: domain::entities::workspace::tests::test_workspace_paths
**問題:** `left: "/path/to/workspace/.tsrc" right: "/path/to/workspace/.tsrc/manifest"`
**原因:** ワークスペースのマニフェストディレクトリパスが期待値と異なる

## 修正方針
各テストの期待値と実装を確認し、適切に修正する。