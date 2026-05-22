# Phase 2-4: Symlink, S3, GoogleDrive SCMタイプ追加

## 実施内容

### ScmType列挙型拡張 (scm_type.rs)
- `Symlink`, `S3`, `GDrive` バリアント追加
- FromStr: symlink|link, s3|aws-s3, gdrive|googledrive|google-drive
- 全capability メソッド更新 (supports_branches, supports_remotes, etc.)
- executable_name: Symlink="", S3="aws", GDrive="rclone"
- is_valid_url_scheme: Symlink=true(全パス), S3="s3://", GDrive=":"含む

### ScmFactory拡張 (scm_factory.rs)
- create_scm: 3タイプともUnsupportedOperation (Httpパターン)
- check_scm_availability: Symlink=always true, S3=aws --version, GDrive=rclone --version
- check_command_availability ヘルパーメソッド追加

### ScmOptions拡張 (manifest.rs)
- S3 { region, endpoint_url } バリアント追加
- GDrive { rclone_remote } バリアント追加

### sync_repositories.rs
- create_or_update_symlink: 相対パス対応、既存リンク検証・再作成
- sync_s3_resource: aws s3 sync + credential_service連携
- sync_gdrive_resource: rclone copy + scm_options対応

### status_check.rs
- S3/GDrive: ディレクトリ存在→Clean (Httpパターン)
- Symlink: リンク有効→Clean、壊れている→Dirty

## 実行コマンド
- `cargo check` — コンパイルエラーなし
- `cargo test --lib -- credential` — 11 passed
- `cargo test --lib -- scm_type` — 7 passed
- `cargo test --lib -- scm_factory` — 2 passed
