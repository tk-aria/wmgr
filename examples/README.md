# wmgr.yaml Configuration Examples

このディレクトリには、異なるSCMシステム（Git、SVN、Perforce）を使用したwmgr.yamlの設定例が含まれています。

## ファイル一覧

### 1. `mixed-scm-workspace.yaml`
**混合SCM環境**
- Git、SVN、Perforceを組み合わせた実際の企業環境を想定
- モダンなGitプロジェクトとレガシーなSVN/P4システムの併用例
- 開発ツール群の異なるSCMからの統合管理

### 2. `svn-only-workspace.yaml`
**SVNのみの環境**
- 従来型のSubversion中心の開発環境
- trunk/branches/tagsの標準的なSVNレイアウト
- 特定のリビジョンやブランチの指定方法

### 3. `perforce-workspace.yaml`
**Perforceエンタープライズ環境**
- 大規模なPerforce環境での設定例
- ストリーム、チェンジリスト、クライアントワークスペースの活用
- SSL接続やTCP接続の例

## 各SCMシステムの特徴

### Git固有の設定
```yaml
- name: git-repo
  url: https://github.com/user/repo.git
  dest: repo
  scm: git
  branch: develop        # ブランチ指定
  remote: upstream      # リモート名指定
  shallow: true         # 浅いクローン
```

### SVN固有の設定
```yaml
- name: svn-repo
  url: https://svn.example.com/repos/project/trunk
  dest: project
  scm: svn
  revision: 1234        # リビジョン指定
  username: user        # 認証情報
  password: pass
```

### Perforce固有の設定
```yaml
- name: p4-repo
  url: perforce://server:1666//depot/project/...
  dest: project
  scm: p4
  client: my-workspace  # クライアント指定
  changelist: 12345     # チェンジリスト指定
  stream: //depot/main  # ストリーム指定
```

## 認証とセキュリティ

実際の運用では、認証情報を直接設定ファイルに記載せず、以下の方法を推奨：

1. **環境変数の使用**
   ```yaml
   username: ${SCM_USERNAME}
   password: ${SCM_PASSWORD}
   ```

2. **外部設定ファイル**
   - `.wmgrrc`ファイルでの認証情報管理
   - システムの認証情報ストアとの連携

3. **SSH鍵認証**（Git/SVN）
   ```yaml
   url: git@github.com:user/repo.git
   url: svn+ssh://user@svn.server.com/repos/project
   ```

## URL形式

### Git URLs
- `https://github.com/user/repo.git`
- `git@github.com:user/repo.git`
- `ssh://git@server.com/repo.git`
- `file:///local/path/to/repo`

### SVN URLs  
- `https://svn.server.com/repos/project`
- `svn://svn.server.com/repos/project`
- `svn+ssh://user@svn.server.com/repos/project`
- `file:///local/svn/repos/project`

### Perforce URLs
- `perforce://server:1666//depot/path/...`
- `p4://server:1666//depot/path/...`
- `ssl:server:1667//depot/path/...`
- `tcp:server:1666//depot/path/...`

## 使用方法

1. 適切な例をコピー
2. 自分の環境に合わせて修正
3. `wmgr.yaml`として保存
4. `wmgr init`でワークスペースを初期化

```bash
cp examples/mixed-scm-workspace.yaml wmgr.yaml
# 設定を編集
wmgr init
wmgr sync
```