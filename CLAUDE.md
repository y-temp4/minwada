# CLAUDE.md

必ず日本語で回答してください。

このファイルは、このリポジトリのコードで作業する際に Claude Code（claude.ai/code）へのガイダンスを提供します。

## 開発コマンド

**開発のためにすべてを起動:**

```bash
just dev  # データベース、バックエンド、フロントエンドを自動リロードで起動
```

**個別サービス:**

```bash
just backend-dev     # cargo-watchでRustバックエンドを起動
just frontend-dev    # Next.jsフロントエンド + API生成ウォッチャーを起動
just db-up          # Docker ComposeでPostgreSQLとMailHogを起動
```

**フロントエンド特有のコマンド:**

```bash
cd frontend
npm run dev                    # Next.js開発サーバー
npm run build                  # 本番ビルド
npm run lint                   # ESLint
npm run typecheck             # TypeScript型チェック
npm run generate-api          # OpenAPI仕様からAPIクライアントを生成
npm run generate-api:watch    # 仕様変更時にAPIクライアントを自動再生成
```

**バックエンド特有のコマンド:**

```bash
cd backend
cargo run                     # サーバー起動
cargo build                  # プロジェクトビルド
cargo test                   # テスト実行
```

**新環境用セットアップ:**

```bash
just setup  # フロントエンド依存関係のインストール、バックエンドビルド、データベース起動
```

## アーキテクチャ概要

これは Rust バックエンドと Next.js フロントエンドを備えた Reddit 風のディスカッションプラットフォームで、自動 API 同期機能を特徴としています。

**バックエンド (ポート 8000):**

- Axum ウェブフレームワークと PostgreSQL/SQLx
- JWT 認証 + OAuth2 (Google)
- utoipa による自動生成 OpenAPI ドキュメント
- メール検証システム (Mailgun/MailHog)

**フロントエンド (ポート 3000):**

- App Router を使用した Next.js 15
- shadcn/ui コンポーネント + Tailwind CSS
- 状態管理に TanStack Query
- Orval による OpenAPI 仕様から自動生成される API クライアント

**主要な統合パターン:**

1. バックエンドが OpenAPI 仕様を自動的に生成
2. Orval が仕様を監視し、TypeScript クライアントを再生成
3. フロントエンドはすべての API 呼び出しに対して完全な型安全性を確保

## 重要なパターン

**API 開発ワークフロー:**

- バックエンドの変更は自動的に `/static/openapi.json` の OpenAPI 仕様を更新
- フロントエンドは `npm run generate-api:watch` で型を再生成
- すべての API 呼び出しは `src/generated/` の生成されたクライアントを通じて型安全

**認証フロー:**

- JWT アクセストークン（15 分有効期限）+ リフレッシュトークン
- フロントエンドで 10 分ごとに自動トークンリフレッシュ
- 認証状態は `src/providers/auth-provider.tsx` の `AuthProvider` で管理

**データベース:**

- マイグレーションは `backend/migrations/` に
- モデルは UUID 主キーを使用
- コンパイル時にチェックされたクエリに SQLx を使用

**ファイル構造の規則:**

- `backend/src/handlers/` - ドメインごとに整理された API エンドポイント
- `backend/src/models/` - データモデルと DTO
- `frontend/src/generated/` - 自動生成 API クライアント（手動編集不可）
- `frontend/src/components/` - 再利用可能な UI コンポーネント

## 開発 URL

- フロントエンド: http://localhost:3000
- バックエンド API: http://localhost:8000
- OpenAPI ドキュメント: http://localhost:8000/swagger-ui/
- MailHog（メールテスト）: http://localhost:8025

## 環境セットアップ

環境ファイルのサンプルをコピー:

```bash
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env.local
```

必要なツール: asdf, Docker Compose, cargo-watch, just
