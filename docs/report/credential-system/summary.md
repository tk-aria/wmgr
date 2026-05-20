# Phase 1: クレデンシャル管理システム

## 実施内容

### 1A. Domain層
- `src/domain/entities/credential.rs` 新規作成
  - CredentialProfile, CredentialFile, CredentialSource, ResolvedCredentials, CredentialError
- `src/domain/entities/manifest.rs` 変更
  - ManifestRepo に `profile: Option<String>` 追加
  - Manifest に `credential_helper: Option<String>` 追加
- `src/domain/entities/mod.rs` に credential モジュール追加

### 1B. Infrastructure層
- `src/infrastructure/credential/mod.rs` 新規作成
- `src/infrastructure/credential/credential_store.rs` 新規作成
  - 4段階クレデンシャル解決: 環境変数 → CLIフラグ → credential.yml → credential_helper
  - credential.yml パーミッション検証 (600)
- `src/infrastructure/credential/credential_helper.rs` 新規作成
  - git credential-helper 互換テキストプロトコル
  - 5秒タイムアウト
- `src/infrastructure/mod.rs` に credential モジュール追加

### 1C. Application層
- `src/application/services/credential_service.rs` 新規作成
  - get_credentials_for_repo: プロファイル解決 + CredentialStore呼び出し + manifest fallback
- `src/application/services/mod.rs` に credential_service モジュール追加

### 1D. CLI層
- `src/presentation/cli/mod.rs` に `--profile` と `--credential-file` グローバルフラグ追加

### 1E. Use Case統合
- `SyncRepositoriesConfig` に credential_profile, credential_file フィールド追加
- `SyncCommand` に credential 情報を伝播
- 子ワークスペース同期にも credential 設定を引き継ぎ

## 実行コマンド
- `cargo check` — コンパイルエラーなし確認
