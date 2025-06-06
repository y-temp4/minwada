# Reddit Clone Sample

Reddit 風のスレッド・コメントシステムのサンプル実装です。
（最終更新: 2025 年 6 月 6 日）

## 技術スタック

### バックエンド

- **Language**: Rust
- **Framework**: axum
- **Database**: PostgreSQL
- **ORM**: sqlx
- **認証**: JWT + OAuth2 (Google、予定)
- **API 仕様**: OpenAPI (utoipa)

### フロントエンド

- **Framework**: Next.js 15+ (App Router)
- **UI**: shadcn/ui + Tailwind CSS
- **API クライアント**: TanStack Query + Orval
- **フォーム**: React Hook Form + Zod

## プロジェクト構成

```
reddit-sample/
├── backend/                 # Rust (axum) バックエンドAPI
│   ├── src/                 # ソースコード
│   │   ├── main.rs          # エントリーポイント
│   │   ├── config.rs        # アプリケーション設定
│   │   ├── models/          # データモデル
│   │   │   ├── auth.rs
│   │   │   ├── threads.rs
│   │   │   └── users.rs
│   │   └── handlers/        # APIハンドラー
│   │       ├── auth.rs
│   │       ├── threads.rs
│   │       └── comments.rs
│   ├── migrations/          # DBマイグレーション
│   ├── database/            # DB設定
│   └── Cargo.toml           # 依存関係
├── frontend/                # Next.js フロントエンド
│   ├── src/                 # ソースコード
│   │   ├── app/             # App Router
│   │   │   ├── layout.tsx
│   │   │   └── page.tsx     # ホームページ
│   │   ├── components/      # UIコンポーネント
│   │   └── lib/             # ユーティリティ
│   ├── generated/           # API自動生成コード
│   └── package.json         # 依存関係
├── justfile                 # タスクランナー
└── README.md                # このファイル
```

## 機能

- ユーザー認証
- スレッド投稿・閲覧
- 木構造コメントシステム
- ユーザープロフィールページ（投稿したスレッド・コメント一覧表示）
- レスポンシブデザイン

## 開発環境セットアップ

### 必要なツール

- asdf
- Docker Compose
- cargo-watch
- just

### インストール手順

#### 事前準備

```bash
$ asdf install
$ cargo install cargo-watch
```

#### バックエンド

```shell
$ cd backend
$ cp .env.example .env
```

#### フロントエンド

```shell
$ cd frontend
$ cp .env.example .env.local
$ npm ci
```

### 開発環境の立ち上げ

```bash
$ just dev
```

### アクセス先

- **フロントエンド**: http://localhost:3000
- **バックエンド API**: http://localhost:8000
- **OpenAPI Docs**: http://localhost:8000/swagger-ui/
- **MailHog**: http://localhost:8025

## 開発ワークフロー

1. バックエンドで API 仕様変更
2. OpenAPI 仕様が自動更新
3. フロントエンドで `npm run generate-api` 実行（もしくは `npm run generate-api:watch` で自動生成）
4. 型安全な API クライアントが再生成

## 実装済み API

### ユーザー関連

- ユーザー登録・ログイン・ログアウト
- Google OAuth 認証
- ユーザープロフィール表示
- ユーザーが投稿したスレッド一覧取得
- ユーザーが投稿したコメント一覧取得
- プロフィール編集

### スレッド関連

- スレッド一覧取得
- スレッド詳細取得
- スレッド作成・編集・削除

### コメント関連

- スレッドのコメント一覧取得
- コメント投稿・編集・削除
- 返信コメント（ネスト構造）

## 注意事項

- 本プロジェクトは学習・デモ用途のサンプル実装です
- 本番環境での使用には追加のセキュリティ対策が必要です
- 継続的に機能追加・改善を行っています
- バグ報告や機能提案は Issue からお願いします
