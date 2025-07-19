# Manifest File Support Enhancement - Summary

## Task Description
- wmgr.yml or manifest.yml 両方のファイル名を探索するように修正
- ファイル拡張子は yml, yaml の両方に対応するように修正

## Executed Operations

### 1. workspace.rs の manifest_file_path() メソッド修正
- ファイル: `src/domain/entities/workspace.rs`
- 修正内容: manifest_file_path() メソッドを拡張し、以下の優先順位でファイルを探索するように変更
  - カレントディレクトリ: wmgr.yml → wmgr.yaml → manifest.yml → manifest.yaml
  - .wmgrディレクトリ: wmgr.yml → wmgr.yaml → manifest.yml → manifest.yaml
- デフォルト値: wmgr.yml

### 2. WorkspaceConfig の default_local() メソッド追加
- ファイル: `src/domain/entities/workspace.rs`
- 追加内容: ローカルファイル用のデフォルトWorkspaceConfigを作成するメソッド

### 3. CLI コマンドファイルの修正
以下のCLIコマンドファイルで、ハードコーディングされたmanifest.yml探索ロジックを削除し、workspace.manifest_file_path()を使用するように変更:

- `src/presentation/cli/commands/apply_manifest.rs`
- `src/presentation/cli/commands/audit.rs`
- `src/presentation/cli/commands/foreach.rs` 
- `src/presentation/cli/commands/log.rs`
- `src/presentation/cli/commands/status.rs`
- `src/presentation/cli/commands/sync.rs`
- `src/presentation/cli/commands/dump_manifest.rs`

### 4. コンパイル確認
- `cargo build` コマンドを実行
- コンパイルエラー解決（Workspace::new()の引数不足を修正）
- 最終的にコンパイル成功

### 5. features.md の更新
該当する項目を `[x]` でマーク完了とした

## Results
- wmgr.yml, wmgr.yaml, manifest.yml, manifest.yaml の4つのファイル形式をサポート
- 優先順位付きのファイル探索機能を実装
- 全CLIコマンドで統一されたマニフェストファイル探索ロジックを適用
- コードの重複を排除し、保守性を向上

## Status: COMPLETED