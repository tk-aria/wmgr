# common/executor.rs 実装時のトラブルシューティング

## エラー1: TaskResultのCloneトレイトの問題

### 問題
```
error[E0277]: the trait bound `TsrcError: Clone` is not satisfied
```

TsrcErrorがCloneトレイトを実装していないため、TaskResult<T>がCloneできない。

### 解決策
TaskResult<T>からCloneトレイトを削除。

## エラー2: execute_with_dependenciesでの参照渡し問題

### 問題
```
error[E0277]: the trait bound `&T: ExecutableTask` is not satisfied
```

参照でExecutableTaskを実装していないため、タスクのイテレーションで問題が発生。

### 解決策
execute_with_dependenciesの引数をVec<&T>に変更し、参照でアクセスするように修正。

## エラー3: cancel_allでの所有権移動問題

### 問題
```
error[E0507]: cannot move out of `*cancel_tx` which is behind a shared reference
```

oneshot::Senderのsendメソッドが所有権を要求するが、参照からムーブできない。

### 解決策
cancel_allでiterからinto_iterに変更し、所有権を取得。

## エラー4: unwrap_errでのDebugトレイト制約

### 問題
```
error[E0277]: `<T as ExecutableTask>::Output` doesn't implement `std::fmt::Debug`
```

ResultのunwrapやCloneにDebugトレイト境界が必要。

### 解決策
ExecutableTaskトレイトのOutputにDebugトレイト境界を追加。

### 実行した操作
1. TaskResult<T>からCloneトレイトを削除
2. ExecutableTaskのOutputにDebug + Clone境界を追加
3. execute_with_dependenciesの実装を修正
4. cancel_allの実装を修正