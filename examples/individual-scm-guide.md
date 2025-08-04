# 各リポジトリ毎のSCM個別設定ガイド

このガイドでは、wmgr.yamlで各リポジトリごとに異なるSCMシステムを個別に設定する方法を詳しく説明します。

## 基本概念

### デフォルト設定とオーバーライド

```yaml
# グローバルデフォルト
defaults:
  scm: git      # 全体のデフォルト
  branch: main
  shallow: false

groups:
  - name: mixed-scm
    repositories:
      # デフォルト設定を使用（Git）
      - name: standard-repo
        url: https://github.com/user/repo.git
        dest: repo
        # scm: git は省略可能（デフォルト使用）
        
      # 個別にSCMをオーバーライド
      - name: svn-repo
        url: https://svn.server.com/repos/project/trunk
        dest: svn-project
        scm: svn  # デフォルトのgitを上書き
        
      # 完全に個別設定
      - name: p4-repo
        url: perforce://p4server:1666//depot/project/...
        dest: p4-project
        scm: p4   # デフォルトのgitを上書き
        client: my-workspace
```

## SCM別の個別設定オプション

### 1. Git個別設定

```yaml
- name: git-custom
  url: https://github.com/user/repo.git
  dest: custom-git
  scm: git
  # Git固有のオプション
  branch: develop          # 特定ブランチ
  remote: upstream         # リモート名
  shallow: true           # 浅いクローン
  extra_options:
    - "--recurse-submodules"
    - "--depth=50"
```

### 2. SVN個別設定

```yaml
- name: svn-custom
  url: https://svn.server.com/repos/project/trunk
  dest: custom-svn
  scm: svn
  # SVN固有のオプション
  revision: 1234          # 特定リビジョン
  username: myuser        # 認証情報
  password: ${SVN_PASS}   # 環境変数使用
  extra_options:
    - "--non-interactive"
    - "--trust-server-cert"
    - "--depth=infinity"
```

### 3. Perforce個別設定

```yaml
- name: p4-custom
  url: perforce://p4server:1666//depot/project/...
  dest: custom-p4
  scm: p4
  # Perforce固有のオプション
  client: my-client-ws    # クライアント名
  changelist: 123456      # 特定チェンジリスト
  stream: //depot/main    # ストリーム
  username: p4user        # 認証情報
  password: ${P4_PASS}    # 環境変数使用
  extra_options:
    - "-f"  # force
    - "-q"  # quiet
```

## 実用的なパターン

### パターン1: プロジェクト別SCM設定

```yaml
groups:
  - name: frontend-projects
    repositories:
      # モダンなReactプロジェクト（Git）
      - name: react-app
        url: https://github.com/company/react-app.git
        dest: frontend/react
        scm: git
        branch: develop
        
      # レガシーなJSPプロジェクト（SVN）
      - name: jsp-legacy
        url: https://svn.company.com/repos/jsp-app/trunk
        dest: frontend/jsp
        scm: svn
        
  - name: backend-services
    repositories:
      # マイクロサービス（Git）
      - name: user-service
        url: git@github.com:company/user-service.git
        dest: backend/user-service
        scm: git
        
      # レガシーモノリス（Perforce）
      - name: legacy-monolith
        url: perforce://p4server:1666//depot/monolith/...
        dest: backend/monolith
        scm: p4
        client: developer-monolith-ws
```

### パターン2: 環境別設定

```yaml
groups:
  - name: multi-environment
    repositories:
      # 開発環境（Git最新）
      - name: app-dev
        url: https://github.com/company/app.git
        dest: app-dev
        scm: git
        branch: develop
        
      # ステージング環境（SVN安定版）
      - name: app-staging
        url: https://svn.company.com/repos/app/branches/release
        dest: app-staging
        scm: svn
        revision: HEAD
        
      # 本番環境（Perforce承認済み）
      - name: app-production
        url: perforce://p4server:1666//depot/app/production/...
        dest: app-prod
        scm: p4
        changelist: 555555  # 承認済みCL
```

### パターン3: チーム・役割別設定

```yaml
groups:
  - name: developer-repos
    repositories:
      # 開発者用（Git）
      - name: source-code
        url: https://github.com/company/source.git
        dest: source
        scm: git
        branch: develop
        
  - name: qa-repos
    repositories:
      # QA用（SVN固定版）
      - name: test-cases
        url: https://svn.company.com/repos/testcases/trunk
        dest: tests
        scm: svn
        revision: 2000  # 固定リビジョン
        
  - name: ops-repos
    repositories:
      # 運用用（Perforce本番）
      - name: deployment-scripts
        url: perforce://p4server:1666//depot/ops/deploy/...
        dest: deploy
        scm: p4
        client: ops-deployment-ws
```

## 認証情報の個別管理

### 環境変数を使用した認証

```yaml
repositories:
  # Git SSH認証
  - name: private-git
    url: git@github.com:company/private.git
    dest: private
    scm: git
    # SSH鍵認証（追加設定不要）
    
  # SVN認証
  - name: secure-svn
    url: https://svn.secure.com/repos/project/trunk
    dest: secure
    scm: svn
    username: ${SVN_USER}
    password: ${SVN_PASSWORD}
    
  # Perforce認証
  - name: enterprise-p4
    url: ssl:p4server:1667//depot/enterprise/...
    dest: enterprise
    scm: p4
    username: ${P4_USER}
    password: ${P4_PASSWORD}
    client: ${P4_CLIENT}
```

### 複数の認証情報

```yaml
repositories:
  # プロジェクトA用SVN
  - name: project-a-svn
    url: https://svn.company.com/repos/project-a/trunk
    dest: project-a
    scm: svn
    username: ${PROJECT_A_SVN_USER}
    password: ${PROJECT_A_SVN_PASS}
    
  # プロジェクトB用SVN（異なる認証）
  - name: project-b-svn
    url: https://svn.company.com/repos/project-b/trunk
    dest: project-b
    scm: svn
    username: ${PROJECT_B_SVN_USER}
    password: ${PROJECT_B_SVN_PASS}
```

## 高度な個別設定

### 同一リポジトリの異なるブランチ/リビジョン

```yaml
repositories:
  # 同じGitリポジトリの異なるブランチ
  - name: main-branch
    url: https://github.com/company/app.git
    dest: app-main
    scm: git
    branch: main
    
  - name: develop-branch
    url: https://github.com/company/app.git
    dest: app-develop
    scm: git
    branch: develop
    
  # 同じSVNリポジトリの異なるリビジョン
  - name: current-svn
    url: https://svn.company.com/repos/app/trunk
    dest: app-current
    scm: svn
    revision: HEAD
    
  - name: stable-svn
    url: https://svn.company.com/repos/app/trunk
    dest: app-stable
    scm: svn
    revision: 1500
```

### 特殊なURL形式

```yaml
repositories:
  # Git - 異なるプロトコル
  - name: https-git
    url: https://github.com/user/repo.git
    dest: https-repo
    scm: git
    
  - name: ssh-git
    url: git@github.com:user/repo.git
    dest: ssh-repo
    scm: git
    
  - name: local-git
    url: file:///local/path/to/repo
    dest: local-repo
    scm: git
    
  # SVN - 異なるプロトコル
  - name: https-svn
    url: https://svn.server.com/repos/project
    dest: https-svn
    scm: svn
    
  - name: ssh-svn
    url: svn+ssh://user@svn.server.com/repos/project
    dest: ssh-svn
    scm: svn
    
  - name: local-svn
    url: file:///local/svn/repos/project
    dest: local-svn
    scm: svn
    
  # Perforce - 異なるプロトコル
  - name: standard-p4
    url: perforce://p4server:1666//depot/project/...
    dest: standard-p4
    scm: p4
    
  - name: ssl-p4
    url: ssl:p4server:1667//depot/project/...
    dest: ssl-p4
    scm: p4
    
  - name: tcp-p4
    url: tcp:p4server:1666//depot/project/...
    dest: tcp-p4
    scm: p4
```

## ベストプラクティス

1. **明示的なSCM指定**: 混在環境では、デフォルトに依存せず明示的に指定
2. **環境変数の活用**: 認証情報は環境変数で管理
3. **一貫した命名**: リポジトリ名にSCMタイプを含めると管理しやすい
4. **グルーピング**: 関連するリポジトリはグループでまとめる
5. **ドキュメント化**: 各リポジトリの用途と設定理由を明記

これにより、複雑な混合SCM環境でも、各リポジトリごとに最適な設定を適用できます。