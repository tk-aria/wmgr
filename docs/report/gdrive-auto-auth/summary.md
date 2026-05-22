# GDrive rclone 自動認証統合 作業報告

## 概要
`wmgr sync` 実行時に GDrive リポジトリが含まれる場合、rclone リモートの認証状態を自動チェックし、未認証ならブラウザ OAuth フローを自動起動する機能を実装。

## 新規ファイル
| ファイル | 内容 |
|---|---|
| `crates/wmgr/src/infrastructure/rclone.rs` | RcloneManager — rclone リモート管理・自動認証 |

## 変更ファイル
| ファイル | 内容 |
|---|---|
| `crates/wmgr/src/infrastructure/mod.rs` | `pub mod rclone;` 追加 |
| `crates/wmgr/src/application/use_cases/sync_repositories.rs` | `sync_gdrive_resource` を RcloneManager 経由に書き換え |

## 認証フロー
1. `rclone --version` で rclone インストール確認
2. `~/.config/wmgr/rclone.conf` にリモート定義が存在するか確認
3. 存在しない場合: `rclone config create <name> drive` で作成
4. `rclone lsf <remote>: --max-depth 0` でトークン有効性チェック
5. 失敗時: `rclone config reconnect <remote>:` でブラウザ OAuth 起動
6. CI環境 (`CI`, `GITHUB_ACTIONS`, `WMGR_HEADLESS`) ではスキップしエラーメッセージを表示

## 設計判断
- wmgr 専用の rclone.conf (`~/.config/wmgr/rclone.conf`) を使用し、ユーザーの既存 rclone 設定を汚さない
- `--config` フラグで全 rclone コマンドに設定ファイルパスを明示
- `resolve_gdrive_source()` を分離し、URL からリモート名を自動推定

## 検証結果
- `cargo check` — OK
- `cargo test -p wmgr --lib` — 206テストパス（+3テスト追加）
