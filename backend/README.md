# minwada Backend

Rust + axum で構築された Reddit 風サービスのバックエンド API です。

## 技術スタック

- **言語**: Rust
- **フレームワーク**: axum
- **データベース**: PostgreSQL
- **ORM**: sqlx
- **認証**: JWT + OAuth2 (Google、予定)
- **API 仕様**: OpenAPI (utoipa)
- **パスワードハッシュ**: argon2

## 機能

- ユーザー認証（メール・パスワード + Google OAuth）
- JWT ベースの認証
- スレッドの CRUD 操作
- 木構造コメントシステム
- OpenAPI 仕様の自動生成
- CORS 対応

## 開発環境セットアップ

### 1. 前提条件

- Rust (`.tool-versions` で指定されたバージョン)
- Docker & Docker Compose
- PostgreSQL (Docker で起動可能)

### 2. データベース起動

```bash
# PostgreSQL + pgAdmin + Redisを起動
docker-compose up -d

# ログ確認
docker-compose logs -f postgres
```

### 3. 環境変数設定

```bash
# 環境変数ファイルをコピー
cp env.example .env

# .envファイルを編集してJWT_SECRETなどを設定
```

### 4. アプリケーション実行

```bash
# 依存関係インストール & 実行
cargo run

# または開発モード（ホットリロード）
cargo watch -x run
```

## API エンドポイント

### 認証

- `POST /api/auth/register` - ユーザー登録
- `POST /api/auth/login` - ログイン
- `POST /api/auth/logout` - ログアウト
- `POST /api/auth/refresh` - トークンリフレッシュ
- `GET /api/auth/google` - Google OAuth 開始
- `GET /api/auth/google/callback` - Google OAuth コールバック

### スレッド

- `GET /api/threads` - スレッド一覧
- `POST /api/threads` - スレッド作成
- `GET /api/threads/{id}` - スレッド詳細
- `PUT /api/threads/{id}` - スレッド更新
- `DELETE /api/threads/{id}` - スレッド削除

### コメント

- `GET /api/threads/{id}/comments` - コメント一覧
- `POST /api/threads/{id}/comments` - コメント作成
- `PUT /api/threads/{id}/comments/{comment_id}` - コメント更新
- `DELETE /api/threads/{id}/comments/{comment_id}` - コメント削除

### ユーザー

- `GET /api/users/me` - 現在のユーザー情報
- `PUT /api/users/me` - プロフィール更新

## API ドキュメント

サーバー起動後、以下の URL で Swagger UI にアクセス可能：

```
http://localhost:8000/swagger-ui/
```

OpenAPI 仕様 JSON：

```
http://localhost:8000/api/openapi.json
```

## データベース管理

### pgAdmin

```
URL: http://localhost:8080
Email: admin@example.com
Password: admin
```

### マイグレーション

```bash
# マイグレーションツールのインストール
cargo install sqlx-cli --no-default-features --features postgres

# マイグレーション実行
sqlx migrate run

# マイグレーション作成
sqlx migrate add <migration_name>
```

## 開発用コマンド

```bash
# コードフォーマット
cargo fmt

# リンター実行
cargo clippy

# テスト実行
cargo test

# 依存関係チェック
cargo audit

# ホットリロード開発
cargo install cargo-watch
cargo watch -x run
```

## 環境変数

| 変数名                     | 説明                                  | デフォルト値                                                        |
| -------------------------- | ------------------------------------- | ------------------------------------------------------------------- |
| `DATABASE_URL`             | PostgreSQL 接続 URL                   | `postgresql://reddit_user:reddit_password@localhost:5433/reddit_db` |
| `SERVER_HOST`              | サーバーホスト                        | `0.0.0.0`                                                           |
| `SERVER_PORT`              | サーバーポート                        | `8000`                                                              |
| `CORS_ORIGIN`              | CORS 許可オリジン                     | `http://localhost:3000`                                             |
| `JWT_SECRET`               | JWT 署名キー                          | **必須設定**                                                        |
| `JWT_EXPIRES_IN`           | JWT の有効期間                        | `15m`                                                               |
| `REFRESH_TOKEN_EXPIRES_IN` | リフレッシュトークンの有効期間        | `7d`                                                                |
| `GOOGLE_CLIENT_ID`         | Google OAuth クライアント ID          | -                                                                   |
| `GOOGLE_CLIENT_SECRET`     | Google OAuth クライアントシークレット | -                                                                   |

## プロジェクト構造

```
backend/
├── src/
│   ├── main.rs              # エントリーポイント
│   ├── config.rs            # 設定管理
│   ├── error.rs             # エラーハンドリング
│   ├── auth.rs              # 認証ユーティリティ
│   ├── middleware.rs        # ミドルウェア
│   ├── utils.rs             # ユーティリティ関数
│   ├── routes.rs            # ルーティング設定
│   ├── models/              # データモデル
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── threads.rs
│   │   ├── comments.rs
│   │   ├── users.rs
│   │   └── common.rs
│   └── handlers/            # リクエストハンドラー
│       ├── mod.rs
│       ├── auth.rs
│       ├── threads.rs
│       ├── comments.rs
│       └── users.rs
├── migrations/              # データベースマイグレーション
├── database/                # DB初期化とDocker設定
├── Cargo.toml              # Rust依存関係
└── README.md               # このファイル
```

## 注意事項

- 本プロジェクトは学習・デモ用途です
- 本番環境では追加のセキュリティ対策が必要です
- JWT_SECRET は本番環境では強力なランダム文字列を使用してください
