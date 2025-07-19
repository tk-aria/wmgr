# Project Implementation Guidelines

## Development Process
- Implement features by referencing features.md from top to bottom
- Continue processing until all items are completed
- Unimplemented items are marked with []
- Mark completed items with [x]
- Compile and check for errors after each individual [] item
- Git add and commit commands can be executed directly without additional confirmation

## Reporting and Documentation
- Always report executed commands in summary.md
- For each task or interruption:
  - Log all operations in docs/report/[task name]/summary.md
  - Commit work differences to git
- If errors occur:
  - Create docs/report/[task name]/troubleshooting.md
  - Document actual error content and resolution steps

## Additional Guidelines
- If encountering the same error 3 times, stop work and summarize the situation

## 日本語 Implementation Guidelines
- features.md の上から順に参照して実装を進めてください
  - 全ての項目を完了するまで処理を続けてください
- 未実装のものは、[]となっています。
- 完了したものは[x]としてチェックをつけてください
- 1つの[]ごとにコンパイルエラーが起きていないか確認するようにして
- gitの add. commitコマンドを確認せずにそのまま実行して良いです
- 同じ問題で3回エラーになった場合は、無理をせず作業を止めて問題の状況を要約してください。

### 作業報告の記録
- 実行したコマンドは、必ず作業内容として summary.mdに報告してください
- 作業中断時又は1つの[]ごとに行った操作ログを全て記した作業内容をdocs/report/タスク名([ ]単位で作成)/summary.mdに追記し、gitへ作業差分をコミットしてください。(gitコマンド実行例) git add .; git commit -m "...")
- エラーが発生した場合は、docs/report/タスク名([ ]単位で作成)/troubleshooting.mdという名称のファイルを作成し、実際のエラー内容とそれを解決した操作をまとめるようにして