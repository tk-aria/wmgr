# tsrc Rust実装 TODOリスト

## プロジェクト設定とディレクトリ構造

- [x] Cargo.tomlの作成（Rust 1.85.0指定、必要なクレート依存関係の定義）
- [x] クリーンアーキテクチャに基づくディレクトリ構造の作成
  - [x] `src/domain/` - ドメインモデルとビジネスロジック
  - [x] `src/application/` - アプリケーションサービスとユースケース
  - [x] `src/infrastructure/` - 外部システムとの連携（Git、ファイルシステム等）
  - [x] `src/presentation/` - CLI インターフェース
  - [x] `src/main.rs` - エントリーポイント
- [x] ローカル開発環境対応
  - [x] manifest.yml サンプルファイルの作成
  - [x] ローカルファイルベースのワークフロー実装
  - [x] .gitignore設定（target/ディレクトリ除外）

## ドメインモデル実装

- [x] `domain/entities/repository.rs` - リポジトリエンティティの実装
  - [x] `Repository` 構造体（dest、remotes、branch、sha1、tag等のフィールド）
  - [x] `Remote` 構造体（name、url）
- [x] `domain/entities/manifest.rs` - マニフェストエンティティの実装
  - [x] `Manifest` 構造体
  - [x] `ManifestRepo` 構造体
  - [x] `Group` 構造体
- [x] `domain/entities/workspace.rs` - ワークスペースエンティティの実装
  - [x] `Workspace` 構造体
  - [x] `WorkspaceConfig` 構造体
- [x] `domain/value_objects/` - 値オブジェクトの実装
  - [x] `GitUrl` - Git URLの検証と正規化
  - [x] `BranchName` - ブランチ名の検証
  - [x] `FilePath` - ファイルパスの検証と操作

## アプリケーション層実装

- [x] `application/use_cases/init_workspace.rs` - ワークスペース初期化ユースケース
  - [x] ローカルマニフェストファイルからの初期化（URLクローン処理を削除）
  - [x] .tsrc/config.yml の作成
  - [x] グループ指定のサポート
  - [x] ローカルファイルベースのワークスペース設定
- [x] `application/use_cases/sync_repositories.rs` - リポジトリ同期ユースケース
  - [x] ワークスペース読み込み処理（ローカルファイルベース）
  - [x] ワークスペース初期化状態の正しい検証
  - [x] マニフェスト更新処理
  - [x] 不足リポジトリのクローン
  - [x] リモート設定の更新
  - [x] ブランチの同期（fast-forward merge）
- [x] `application/use_cases/status_check.rs` - ステータス確認ユースケース
  - [x] 各リポジトリのGitステータス取得
  - [x] ダーティ状態の検出
  - [x] ブランチ差分の確認
- [x] `application/use_cases/foreach_command.rs` - foreach実行ユースケース
  - [x] 環境変数の設定
  - [x] 並列実行のサポート
- [x] `application/services/manifest_service.rs` - マニフェスト管理サービス
  - [x] マニフェストのパースと検証
  - [x] グループのフィルタリング
  - [x] Deep Manifest/Future Manifestのサポート

## インフラストラクチャ層実装

- [x] `infrastructure/git/repository.rs` - Gitリポジトリ操作の実装
  - [x] `git2`クレートを使用したGit操作のラッパー
  - [x] clone、fetch、checkout、reset等の実装
- [x] `infrastructure/git/remote.rs` - Gitリモート操作の実装
  - [x] リモートの追加、更新、削除
  - [x] URLの検証と正規化
- [x] `infrastructure/git/mod.rs` - Git module definition
- [x] `infrastructure/filesystem/config_store.rs` - 設定ファイル管理
  - [x] YAML形式の設定ファイルの読み書き
  - [x] スキーマ検証
  - [x] ローカルファイルパス検証対応（URL検証の削除）
- [x] `infrastructure/filesystem/manifest_store.rs` - マニフェストファイル管理
  - [x] manifest.yml の読み込みと解析
  - [x] ファイルシステム操作（copy、symlink）
- [x] `infrastructure/process/command_executor.rs` - 外部コマンド実行
  - [x] プロセス生成と管理
  - [x] 並列実行のサポート
  - [x] 環境変数ハンドリング
  - [x] タイムアウトサポート
  - [x] 出力キャプチャ
- [x] `infrastructure/process/mod.rs` - Process module definition

## プレゼンテーション層実装

- [x] `presentation/cli/mod.rs` - CLIメインモジュール
  - [x] `clap`クレートを使用したコマンドライン引数パーサー
- [x] `presentation/cli/commands/init.rs` - initコマンドの実装
  - [x] 引数: manifest_path、--group、--force等（ローカルマニフェスト対応）
  - [x] ローカルファイルからのマニフェスト読み込み
  - [x] リモートダウンロード処理の削除
- [x] `presentation/cli/commands/sync.rs` - syncコマンドの実装
  - [x] 引数: --group、--force、--no-correct-branch等
- [x] `presentation/cli/commands/status.rs` - statusコマンドの実装
  - [x] 引数: --branch、--compact等
- [x] `presentation/cli/commands/foreach.rs` - foreachコマンドの実装
  - [x] 引数: command、--group、--parallel等
- [x] `presentation/cli/commands/log.rs` - logコマンドの実装
- [x] `presentation/cli/commands/dump_manifest.rs` - dump-manifestコマンドの実装
- [x] `presentation/cli/commands/apply_manifest.rs` - apply-manifestコマンドの実装
- [x] `presentation/ui/display.rs` - UI表示ヘルパー
  - [x] カラー出力サポート
  - [x] プログレス表示
  - [x] エラーメッセージフォーマット

## 共通機能実装

- [x] `common/error.rs` - エラー型の定義
  - [x] `thiserror`クレートを使用したエラー型定義
  - [x] Git操作エラー、ファイルシステムエラー、設定エラー等
- [x] `common/result.rs` - Result型のエイリアス定義
- [x] `common/executor.rs` - タスク実行フレームワーク
  - [x] 並列実行サポート
  - [x] プログレス表示
  - [x] エラーハンドリング

## テスト実装

- [x] 単体テストの作成
  - [x] 各モジュールに対する単体テスト
  - [x] モックを使用した依存関係の分離
- [x] 統合テストの作成
  - [x] `tests/`ディレクトリにエンドツーエンドテスト
  - [x] テスト用のGitサーバーモック実装
- [x] テストヘルパーの実装
  - [x] テスト用ワークスペースの作成
  - [x] テスト用マニフェストの生成

## ビルドとパッケージング

- [x] CI/CD設定
  - [x] GitHub Actionsワークフロー設定
  - [x] cargo fmt、cargo clippy、cargo testの自動実行
- [x] リリースビルド設定
  - [x] クロスプラットフォームビルド（Linux、macOS、Windows）
  - [x] バイナリの最適化設定

## ドキュメント

- [x] README.mdの作成
  - [x] インストール手順
  - [x] 使用方法
  - [x] 設定例
- [x] API ドキュメントの生成設定
  - [x] cargo docの設定
- [x] ユーザーガイドの作成
  - [x] 各コマンドの詳細説明
  - [x] ユースケース例

## 性能最適化とメモリ管理

- [x] 並列処理の最適化
  - [x] `tokio`または`rayon`を使用した並列化
  - [x] スレッドプールの適切な設定
- [x] メモリ効率の改善
  - [x] 大規模リポジトリ対応
  - [x] ストリーミング処理の実装

## 互換性とマイグレーション

- [ ] Python版tsrcとの設定ファイル互換性確保
- [ ] マイグレーションスクリプトの作成
- [ ] 後方互換性の維持

## セキュリティ

- [x] 依存関係の脆弱性チェック
  - [x] cargo-auditの導入
- [x] 入力検証の強化
  - [x] URLインジェクション対策
  - [x] ファイルパストラバーサル対策
- [x] wmgr.yml or manifest.yml 両方のファイル名を探索するように修正
- [x] ファイル拡張子は yml, yaml の両方に対応するように修正

## GitHub Actions CI/CD自動リリース

- [x] GitHub Actionsワークフローでマルチプラットフォーム対応リリースビルドの作成
  - [x] `.github/workflows/release.yml` の作成
    - [x] Windows用ビルド（x86_64-pc-windows-gnu）
    - [x] macOS用ビルド（x86_64-apple-darwin, aarch64-apple-darwin）
    - [x] Linux用ビルド（x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu）
    - [x] クロスコンパイル環境設定
    - [x] バイナリの圧縮・アーカイブ化
    - [x] GitHub Releasesへの自動アップロード
    - [x] リリースタグ（v*）のプッシュでワークフロー実行
- [x] actコマンドを使用したローカルワークフローテスト
  - [x] act環境のセットアップ
  - [x] ローカルでのワークフロー実行確認
- [x] リリースワークフローの手動実行機能追加
  - [x] workflow_dispatchトリガーの追加
  - [x] タグ入力パラメータの設定
  - [x] 手動実行時の動的タグ名対応

## インストールスクリプトの作成

- [x] wmgr用の公式インストールスクリプトの作成
  - [x] `scripts/install.sh` - メインインストールスクリプト
    - [x] OS検出機能（Linux、macOS、Windows対応）
    - [x] アーキテクチャ検出機能（amd64、arm64、arm対応）  
    - [x] 最新バージョン自動取得機能
    - [x] GitHub Releasesからのバイナリダウンロード
    - [x] バイナリの実行権限設定
    - [x] インストールパスの設定（/usr/local/bin）
    - [x] バージョン指定インストール対応
  - [x] 使用方法のドキュメント化
    - [x] `curl -sSLf https://get.wmgr.sh | sh` でのインストール
    - [x] `curl -sSLf https://get.wmgr.sh | WMGR_VERSION=v1.0.0 sh` でのバージョン指定
  - [x] エラーハンドリングの実装
    - [x] サポートされていないOS/アーキテクチャの適切な検出
    - [x] ダウンロード失敗時の適切なエラーメッセージ
    - [x] 権限不足時の適切な案内

### サンプルコード: `scripts/install.sh`

```bash
#!/bin/sh

set -e

if [ -n "${DEBUG}" ]; then
  set -x
fi

# デフォルト設定
DEFAULT_INSTALL_PATH="/usr/local/bin"
WMGR_REPO="tk-aria/wmgr"

# 最新バージョンを取得
_wmgr_latest() {
  curl -sSLf "https://api.github.com/repos/${WMGR_REPO}/releases/latest" | \
    grep '"tag_name":' | \
    sed -E 's/.*"([^"]+)".*/\1/'
}

# OS検出
_detect_os() {
  os="$(uname -s)"
  case "$os" in
    Linux) echo "linux" ;;
    Darwin) echo "darwin" ;;
    CYGWIN*|MINGW*|MSYS*) echo "windows" ;;
    *) echo "Unsupported operating system: $os" 1>&2; return 1 ;;
  esac
  unset os
}

# アーキテクチャ検出
_detect_arch() {
  arch="$(uname -m)"
  case "$arch" in
    amd64|x86_64) echo "x86_64" ;;
    arm64|aarch64) echo "aarch64" ;;
    armv7l|armv8l|arm) echo "armv7" ;;
    *) echo "Unsupported processor architecture: $arch" 1>&2; return 1 ;;
  esac
  unset arch
}

# バイナリ名を決定
_get_binary_name() {
  os="$1"
  case "$os" in
    windows) echo "wmgr.exe" ;;
    *) echo "wmgr" ;;
  esac
}

# ダウンロードURL生成
_download_url() {
  local version="$1"
  local os="$2"
  local arch="$3"
  
  # バイナリファイル名: wmgr-{version}-{os}-{arch}.tar.gz
  local archive_name="wmgr-${version}-${os}-${arch}.tar.gz"
  echo "https://github.com/${WMGR_REPO}/releases/download/${version}/${archive_name}"
}

# インストール実行
main() {
  # バージョン決定
  if [ -z "${WMGR_VERSION}" ]; then
    WMGR_VERSION=$(_wmgr_latest)
    if [ -z "${WMGR_VERSION}" ]; then
      echo "Failed to get latest version" 1>&2
      return 1
    fi
  fi

  # インストールパス決定
  wmgr_install_path="${WMGR_INSTALL_PATH:-$DEFAULT_INSTALL_PATH}"
  
  # プラットフォーム検出
  wmgr_os="$(_detect_os)"
  wmgr_arch="$(_detect_arch)"
  wmgr_binary="$(_get_binary_name "$wmgr_os")"
  
  # ダウンロードURL生成
  wmgr_download_url="$(_download_url "$WMGR_VERSION" "$wmgr_os" "$wmgr_arch")"

  echo "Installing wmgr ${WMGR_VERSION} for ${wmgr_os}/${wmgr_arch}..."
  echo "Download URL: $wmgr_download_url"

  # インストールディレクトリ作成
  if [ ! -d "$wmgr_install_path" ]; then
    echo "Creating install directory: $wmgr_install_path"
    mkdir -p -- "$wmgr_install_path"
  fi

  # 一時ディレクトリ作成
  tmp_dir=$(mktemp -d)
  trap 'rm -rf "$tmp_dir"' EXIT

  # アーカイブダウンロード
  echo "Downloading wmgr archive..."
  curl -sSLf "$wmgr_download_url" -o "$tmp_dir/wmgr.tar.gz"

  # アーカイブ展開
  echo "Extracting wmgr archive..."
  tar -xzf "$tmp_dir/wmgr.tar.gz" -C "$tmp_dir"

  # バイナリ配置
  echo "Installing wmgr to $wmgr_install_path/$wmgr_binary"
  cp "$tmp_dir/$wmgr_binary" "$wmgr_install_path/$wmgr_binary"
  chmod 755 -- "$wmgr_install_path/$wmgr_binary"

  echo ""
  echo "✅ wmgr ${WMGR_VERSION} has been successfully installed!"
  echo ""
  echo "The wmgr binary is installed at: $wmgr_install_path/$wmgr_binary"
  echo ""
  echo "To get started, run:"
  echo "  wmgr --help"
  echo ""
  echo "For more information, visit: https://github.com/${WMGR_REPO}"
}

main "$@"
```

### 使用例:

```bash
# 最新バージョンをインストール
curl -sSLf https://get.wmgr.sh | sh

# 特定バージョンをインストール  
curl -sSLf https://get.wmgr.sh | WMGR_VERSION=v1.0.0 sh

# カスタムインストールパスを指定
curl -sSLf https://get.wmgr.sh | WMGR_INSTALL_PATH=/usr/bin sh

# デバッグモードで実行
curl -sSLf https://get.wmgr.sh | DEBUG=1 sh
```