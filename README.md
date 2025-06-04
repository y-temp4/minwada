# Reddit Clone Sample

Reddit 風のスレッド・コメントシステムのサンプル実装です。

## 技術スタック

### バックエンド

- **Language**: Rust
- **Framework**: axum
- **Database**: PostgreSQL
- **ORM**: sqlx
- **認証**: JWT + OAuth2 (Google)
- **API 仕様**: OpenAPI (utoipa)

### フロントエンド

- **Framework**: Next.js 15+ (App Router)
- **UI**: shadcn/ui + Tailwind CSS
- **状態管理**: TanStack Query
- **API Client**: Orval (OpenAPI から自動生成)
- **フォーム**: React Hook Form + Zod

## プロジェクト構成

```
reddit-sample/
├── backend/              # Rust (axum) バックエンドAPI
│   ├── src/             # ソースコード
│   ├── migrations/      # データベースマイグレーション
│   ├── database/        # DB初期化ファイル & Docker Compose
│   └── Cargo.toml       # Rust依存関係
├── frontend/            # Next.js フロントエンド
│   ├── src/            # ソースコード
│   ├── generated/      # Orval生成APIクライアント
│   └── package.json    # Node.js依存関係
└── README.md           # このファイル
```

## 機能

- ユーザー認証（パスワード + Google OAuth）
- スレッド投稿・閲覧
- 木構造コメントシステム
- レスポンシブデザイン

## 開発環境セットアップ

### 1. バックエンド

```bash
cd backend

# 環境変数設定
cp .env.example .env

# データベース起動
docker-compose up -d

# Rust依存関係インストール & 実行
cargo run
```

### 2. フロントエンド

```bash
cd frontend

# 環境変数設定
cp .env.example .env

# 依存関係インストール
npm install

# shadcn/ui初期化
npx shadcn-ui@latest init

# APIクライアント生成 (バックエンドが起動している必要がある)
npm run generate-api

# 開発サーバー起動
npm run dev
```

### 3. アクセス

- **フロントエンド**: http://localhost:3000
- **バックエンド API**: http://localhost:8000
- **OpenAPI Docs**: http://localhost:8000/swagger-ui/
- **pgAdmin**: http://localhost:8080

## 開発ワークフロー

1. バックエンドで API 仕様変更
2. OpenAPI 仕様が自動更新
3. フロントエンドで `npm run generate-api` 実行
4. 型安全な API クライアントが再生成

## 注意事項

- 本プロジェクトは学習・デモ用途のサンプル実装です
- 本番環境での使用には追加のセキュリティ対策が必要です
