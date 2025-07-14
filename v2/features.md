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