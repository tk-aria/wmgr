# tsrc → wmgr リネーム 作業報告

## 概要
ソースコード・テスト・ドキュメント内の旧名称 `tsrc` を `wmgr` にリネーム。

## 変更箇所

### ソースコード
- `src/domain/entities/workspace.rs` — `.tsrc` → `.wmgr`、`tsrc_dir()` → `wmgr_dir()`
- `src/application/use_cases/init_workspace.rs` — `.tsrc` → `.wmgr`
- `src/domain/value_objects/file_path.rs` — 許可リスト `.tsrc` → `.wmgr`
- `src/common/error.rs` — `TsrcError` → `WmgrError`
- `src/common/result.rs` — `TsrcResult` → `WmgrResult`、`ok_or_tsrc` → `ok_or_wmgr`、`map_tsrc_err` → `map_wmgr_err`
- `src/common/executor.rs` — 型名リネーム
- `src/infrastructure/http.rs` — 型名リネーム
- `src/lib.rs` — 公開型名リネーム
- `build.rs` — `TSRC_VERSION` → `WMGR_VERSION`

### テスト
- `tests/integration_command_execution.rs`
- `tests/integration_workspace_lifecycle.rs`
- `tests/test_helpers_integration.rs`
- `tests/common/assertion_helpers.rs`
- `tests/common/test_helpers.rs`
- `tests/common/test_fixtures.rs`

### ドキュメント・設定
- `README.md`, `USER_GUIDE.md`
- `docs/API_OVERVIEW.md`, `docs/EXAMPLES.md`, `docs/MODULES.md`, `docs/index.html`
- `features.md`
- `examples/command_executor_usage.rs`
- `.github/workflows/ci.yml`
- `scripts/build-releases.sh`, `scripts/dev-build.sh`

## 検証結果
- `cargo check` — OK
- `cargo test --lib` — 207テスト全パス
