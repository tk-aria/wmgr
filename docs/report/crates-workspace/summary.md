# crates/ ワークスペース化 作業報告

## 概要
単一クレート構造を Cargo workspace に分割し、crates/ ディレクトリ以下に配置。

## 構成
```
crates/
├── wmgr/          # ライブラリ (domain + application + infrastructure + common)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── domain/
│       ├── application/
│       ├── infrastructure/
│       └── common/
└── wmgr-cli/      # バイナリ (main.rs + presentation/)
    ├── Cargo.toml
    ├── build.rs
    └── src/
        ├── main.rs
        └── presentation/
```

## 変更内容
1. ルート `Cargo.toml` を `[workspace]` 定義に変更
2. `crates/wmgr/Cargo.toml` — ライブラリクレート作成
3. `crates/wmgr-cli/Cargo.toml` — バイナリクレート作成（wmgr依存）
4. presentation/ 内の `crate::` 参照を `wmgr::` に変更
5. `include_str!` パスを crates/ 配置に合わせて修正
6. `build.rs` の `.git/` パスを修正
7. 旧 `src/` と `build.rs` を削除

## 設計判断
- application層がinfrastructure層を直接参照しているため、3クレート分割は循環依存が発生
- 2クレート分割（lib + cli）を採用

## 検証結果
- `cargo check` — OK
- `cargo test -p wmgr --lib` — 203テストパス
- `cargo test -p wmgr-cli` — 4テストパス
- 合計207テスト全パス
