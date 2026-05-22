# PR #2 レビュー指摘修正 作業報告

## 概要
PR #2 のコードレビューで指摘された項目 #2〜#8 を修正。

## 修正内容

| # | ファイル | 修正内容 |
|---|---|---|
| 2 | `credential_store.rs` | パーミッション不正時の `eprintln!` 警告を `CredentialError::PermissionDenied` エラーに変更（セキュリティ強化） |
| 3 | `s3.rs` | `std::fs::create_dir_all` / `std::fs::write` を `tokio::fs` に置換（async関数内のブロッキングI/O解消） |
| 4 | `credential.rs` | `merge_from()` のフィールド毎の繰り返しをマクロ `fill!` で簡潔化 |
| 5 | `scm_type.rs:21` | S3のdocコメントを "via aws s3 CLI" → "via AWS SDK for Rust" に更新 |
| 6 | `features.md` | タイトル "tsrc" → "wmgr"、"Python版tsrc" → "Python版wmgr(tsrc)" に修正 |
| 7 | `Cargo.toml` (両クレート) + README | `rust-version` を `1.70.0` → `1.78.0` に統一（AWS SDK要件に合わせて）、README も Rust 1.78+ に更新 |
| 8 | `build.rs` | 未使用の `WMGR_VERSION` 環境変数設定を削除 |

## 検証結果
- `cargo check` — OK（エラーなし）
- `cargo test -p wmgr --lib` — 203テストパス
- `cargo test -p wmgr-cli` — 4テストパス
- 合計207テスト全パス
