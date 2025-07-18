# common/result.rs 実装時のトラブルシューティング

## エラー1: async_helpers::wrap_async関数のサイズ問題

### 問題
```
error[E0277]: the size for values of type `dyn StdError + std::marker::Send + Sync` cannot be known at compilation time
```

Box<dyn Error + Send + Sync>のサイズが分からないため、TsrcError::internal_error_with_sourceで使用できない。

### 解決策
wrap_async関数を削除し、より単純なasync_helpersモジュールに変更。

## エラー2: with_filesystem_error関数のdowncastメソッド問題

### 問題
```
error[E0599]: no method named `downcast` found for type parameter `E`
```

ジェネリック型Eに対してdowncastメソッドが使用できない。

### 解決策
with_filesystem_error関数を簡略化し、std::io::Errorを直接受け取るように変更。

### 実行した操作
1. async_helpers::wrap_async関数を削除
2. with_filesystem_error関数を修正
3. 該当するテストコードを修正