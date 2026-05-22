# wmgr

**Workspace Manager** — 複数リポジトリのワークスペースを管理する高速CLIツール

[![CI](https://github.com/tk-aria/wmgr/actions/workflows/ci.yml/badge.svg)](https://github.com/tk-aria/wmgr/actions/workflows/ci.yml)
[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD--3--Clause-blue.svg)](LICENSE)

> [English README](README.md)

---

## 概要

`wmgr` はYAMLマニフェストひとつで、複数リポジトリの定義・同期・一括操作を実現します。**Git, SVN, Perforce, Mercurial, HTTPアーカイブ, S3, Google Drive, シンボリックリンク** に対応し、統一されたワークフローで管理できます。

```
wmgr.yml          wmgr sync           ワークスペース
+-----------+      ---------->      +------------------+
| repos:    |                       | frontend/        |
|  - frontend|                      | backend/         |
|  - backend |                      | shared-assets/   |
|  - assets  |                      | design-files/    |
+-----------+                       +------------------+
```

## 特徴

- **マルチSCM対応** — Git, SVN, Perforce, Mercurial, HTTP, S3, Google Drive, Symlink
- **グループフィルタリング** — リポジトリをグループに整理し、サブセットごとに操作
- **並列実行** — 複数リポジトリに対してコマンドを同時実行
- **クレデンシャル管理** — 4段階の認証情報解決（環境変数, マニフェスト, プロファイル, システム）
- **再帰的ワークスペース** — ネストされたワークスペースの子マニフェスト自動検出
- **複数出力形式** — Text, JSON, YAML（CI/スクリプト連携に便利）

## インストール

### クイックインストール（推奨）

```bash
curl -sSLf https://raw.githubusercontent.com/tk-aria/wmgr/main/scripts/install.sh | sh
```

OS・アーキテクチャを自動検出し、適切なバイナリを `/usr/local/bin` にインストールします。

### オプション

```bash
# バージョン指定
curl -sSLf https://raw.githubusercontent.com/tk-aria/wmgr/main/scripts/install.sh | WMGR_VERSION=v1.0.0 sh

# インストール先を変更
curl -sSLf https://raw.githubusercontent.com/tk-aria/wmgr/main/scripts/install.sh | WMGR_INSTALL_PATH=$HOME/.local/bin sh
```

### 手動ダウンロード

[Releases](https://github.com/tk-aria/wmgr/releases) からダウンロードし、`PATH` の通った場所に配置：

| プラットフォーム | ファイル |
|---|---|
| Linux x86_64 | `wmgr-linux-x86_64.tar.gz` |
| Linux ARM64 | `wmgr-linux-aarch64.tar.gz` |
| macOS Intel | `wmgr-darwin-x86_64.tar.gz` |
| macOS Apple Silicon | `wmgr-darwin-aarch64.tar.gz` |
| Windows x86_64 | `wmgr-windows-x86_64.tar.gz` |

```bash
# 例: macOS Apple Silicon
curl -L https://github.com/tk-aria/wmgr/releases/latest/download/wmgr-darwin-aarch64.tar.gz | tar xz
sudo mv wmgr /usr/local/bin/
```

### ソースからビルド

```bash
git clone https://github.com/tk-aria/wmgr.git
cd wmgr
cargo build --release
sudo cp target/release/wmgr /usr/local/bin/
```

**要件:** Rust 1.70+, Git 2.0+

---

## クイックスタート

**1. マニフェストを作成** (`wmgr.yml`):

```yaml
repos:
  - dest: frontend
    url: https://github.com/example/frontend.git
    branch: main
    groups: [web]

  - dest: backend
    url: https://github.com/example/backend.git
    branch: main
    groups: [api]

  - dest: shared-assets
    url: s3://my-bucket/assets
    scm: s3
    scm_options:
      type: S3
      region: ap-northeast-1
```

**2. 全リポジトリを同期:**

```bash
wmgr sync
```

**3. ステータスを確認:**

```bash
wmgr status
```

---

## コマンド一覧

| コマンド | 説明 |
|---|---|
| `wmgr init <manifest>` | マニフェストからワークスペースを初期化 |
| `wmgr sync` | 全リポジトリを同期 |
| `wmgr status` | リポジトリのステータスを表示 |
| `wmgr foreach <cmd>` | 各リポジトリでコマンドを実行 |
| `wmgr log` | リポジトリ横断のコミットログを表示 |
| `wmgr dump-manifest` | 現在のマニフェストを出力 |
| `wmgr apply-manifest <file>` | 新しいマニフェストを適用 |
| `wmgr audit` | ワークスペースのセキュリティ監査 |

### 使用例

```bash
# webグループのみ同期
wmgr sync --group web

# 全リポジトリで並列テスト実行
wmgr foreach "make test" --parallel

# ステータスをJSON形式で出力（CI向け）
wmgr status --output json

# 強制再同期
wmgr sync --force
```

---

## マニフェスト形式

### 基本（フラットリスト）

```yaml
repos:
  - dest: my-app
    url: https://github.com/org/my-app.git
    branch: main
```

### マルチSCMワークスペース

```yaml
repos:
  # Gitリポジトリ
  - dest: backend
    url: https://github.com/org/backend.git
    branch: develop
    groups: [core]

  # HTTPアーカイブダウンロード（自動展開）
  - dest: tools/sdk
    url: https://releases.example.com/sdk-v2.0.tar.gz
    scm: http

  # S3バケット同期
  - dest: assets/textures
    url: s3://game-assets/textures
    scm: s3
    scm_options:
      type: S3
      region: us-east-1

  # Google Drive（rclone経由）
  - dest: design-files
    url: "gdrive-remote:Design/ProjectX"
    scm: gdrive
    scm_options:
      type: GDrive
      rclone_remote: gdrive-remote

  # ローカルシンボリックリンク
  - dest: shared-config
    url: /opt/shared/config
    scm: symlink
```

### リポジトリオプション

| フィールド | 必須 | 説明 |
|---|---|---|
| `dest` | Yes | ローカルディレクトリ名 |
| `url` | Yes | リポジトリURL またはパス |
| `branch` | No | チェックアウトするブランチ（デフォルト: `main`） |
| `groups` | No | 所属グループ |
| `scm` | No | SCMタイプ: `git`, `svn`, `p4`, `hg`, `http`, `s3`, `gdrive`, `symlink` |
| `scm_options` | No | SCM固有の設定 |
| `tag` | No | 特定のタグ |
| `sha1` | No | 特定のコミットハッシュ |
| `credential` | No | クレデンシャルプロファイル名 |

---

## クレデンシャル管理

wmgr は4段階のクレデンシャル解決をサポートしています：

1. **環境変数** — `WMGR_AWS_ACCESS_KEY_ID` など
2. **マニフェスト直書き** — `wmgr.yml` に直接記述
3. **プロファイル参照** — `~/.config/wmgr/credential.yml`
4. **システム認証ヘルパー** — git credential helper, AWS CLI設定など

```yaml
# ~/.config/wmgr/credential.yml
profiles:
  my-s3:
    type: S3
    access_key_id: AKIA...
    secret_access_key: ...
    region: ap-northeast-1
```

```yaml
# wmgr.yml
repos:
  - dest: assets
    url: s3://my-bucket/assets
    scm: s3
    credential: my-s3
```

---

## CI/CD連携

### GitHub Actions

```yaml
- name: Install wmgr
  run: |
    curl -sSLf https://raw.githubusercontent.com/tk-aria/wmgr/main/scripts/install.sh | sh

- name: Sync workspace
  run: wmgr sync

- name: Run tests
  run: wmgr foreach "make test" --parallel
```

---

## プロジェクト構成

```
wmgr/
├── crates/
│   ├── wmgr/          # ライブラリクレート（domain, application, infrastructure）
│   └── wmgr-cli/      # バイナリクレート（CLIインターフェース）
├── config/            # 埋め込みテンプレート
├── scripts/           # インストール・ビルドスクリプト
├── templates/         # ユーザー向けYAMLテンプレート
├── tests/             # 統合テスト
└── docs/              # ドキュメント
```

## 開発

```bash
git clone https://github.com/tk-aria/wmgr.git
cd wmgr
cargo build            # 開発ビルド
cargo test             # 全テスト実行
cargo clippy           # リント
cargo fmt              # フォーマット
```

## ライセンス

[BSD-3-Clause](LICENSE)

## 謝辞

- [tsrc](https://github.com/TankerHQ/tsrc)（Python実装）にインスパイア
- Rustエコシステムで構築
