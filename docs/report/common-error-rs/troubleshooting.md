# common/error.rs 実装時のトラブルシューティング

## エラー1: anyhow::Error のトレイト境界エラー

### 問題
```
error[E0277]: the trait bound `anyhow::Error: StdError` is not satisfied
```

anyhow::ErrorがStdErrorトレイトを実装していないため、TsrcError::internal_error_with_sourceで使用できない。

### 解決策
anyhow::ErrorからTsrcErrorへの変換を修正する。anyhow::Errorを文字列に変換してから使用する。

### 実行した操作
1. anyhow::ErrorからTsrcErrorへの変換実装を修正
2. anyhow::Errorを文字列として扱うように変更