<p align="center">
  <a href="https://minwada.com">
    <img src="./docs/images/logo.png" alt="みんなの話題" width="300">
  </a>
</p>

# 🗣️ [みんなの話題](https://minwada.com)

[![Rust](https://img.shields.io/badge/rust-1.87.0-orange.svg?logo=rust)](https://www.rust-lang.org)
[![Next.js](https://img.shields.io/badge/next.js-15+-black.svg?logo=next.js)](https://nextjs.org/)

[English README](./docs/README.en.md)

> Reddit 風のスレッド・コメントシステムのサンプル実装です。自由に話題を投稿し、コメントを通じてコミュニケーションを楽しめます。

> [!WARNING]
> 本 README を含むプロジェクト内のコードは大半が AI により生成されたものであり、必ずしも正しい内容とは限らない点にご注意ください。

## ✨ 機能

- 👤 ユーザー認証（JWT）
- 📝 スレッド投稿・閲覧
- 💬 木構造コメントシステム
- 👨‍💻 ユーザープロフィールページ（投稿したスレッド・コメント一覧表示）
- 📱 レスポンシブデザイン

## 🛠️ 技術スタック

### バックエンド

- **Language**: [Rust](https://www.rust-lang.org/) 🦀
- **Framework**: [axum](https://github.com/tokio-rs/axum) ⚡
- **Database**: [PostgreSQL](https://www.postgresql.org/) 🐘
- **ORM**: [sqlx](https://github.com/launchbadge/sqlx) 📊
- **認証**: JWT + OAuth2 (Google、予定) 🔐
- **API 仕様**: [OpenAPI](https://www.openapis.org/) ([utoipa](https://github.com/juhaku/utoipa)) 📚

### フロントエンド

- **Framework**: [Next.js](https://nextjs.org/) 15+ (App Router) ⚛️
- **UI**: [shadcn/ui](https://ui.shadcn.com/) + [Tailwind CSS](https://tailwindcss.com/) 🎨
- **API クライアント**: [TanStack Query](https://tanstack.com/query) + [Orval](https://orval.dev/) 🔄
- **フォーム**: [React Hook Form](https://react-hook-form.com/) + [Zod](https://zod.dev/) 📋

## 📂 プロジェクト構成

```
minwada/
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

## 🚀 開発環境セットアップ

### 必要なツール

- [asdf](https://asdf-vm.com/) - バージョン管理
- [Docker Compose](https://docs.docker.com/compose/) - コンテナ管理
- [cargo-watch](https://crates.io/crates/cargo-watch) - Rust ホットリロード
- [just](https://just.systems/) - タスクランナー

### インストール手順

#### 1️⃣ 事前準備

```bash
$ asdf install
$ cargo install cargo-watch
```

#### 2️⃣ バックエンド

```shell
$ cd backend
$ cp .env.example .env  # 環境変数を設定
```

#### 3️⃣ フロントエンド

```shell
$ cd frontend
$ cp .env.example .env.local  # 環境変数を設定
$ npm ci  # 依存関係をインストール
```

### 開発環境の立ち上げ

```bash
$ just dev  # バックエンドとフロントエンドの両方が起動します
```

### アクセス先

- 🌐 **フロントエンド**: http://localhost:3000
- 🔌 **バックエンド API**: http://localhost:8000
- 📘 **OpenAPI Docs**: http://localhost:8000/swagger-ui/
- 📧 **MailHog**: http://localhost:8025

## 💻 開発ワークフロー

1. バックエンドで API 仕様変更
2. OpenAPI 仕様が自動更新
3. フロントエンドで `npm run generate-api` 実行（もしくは `npm run generate-api:watch` で自動生成）
4. 型安全な API クライアントが再生成

## 🔍 実装済み API

<details>
<summary>👤 ユーザー関連</summary>

- ユーザー登録・ログイン・ログアウト
- ユーザープロフィール表示
- ユーザーが投稿したスレッド一覧取得
- ユーザーが投稿したコメント一覧取得
- プロフィール編集
</details>

<details>
<summary>📋 スレッド関連</summary>

- スレッド一覧取得
- スレッド詳細取得
- スレッド作成・削除
</details>

<details>
<summary>💬 コメント関連</summary>

- スレッドのコメント一覧取得
- コメント投稿・削除
- 返信コメント（ネスト構造）
</details>

## ⚠️ 注意事項

- 本プロジェクトは学習・デモ用途のサンプル実装です
- 実際の運用にはセキュリティやパフォーマンスの考慮が必要です
- 継続的に機能追加・改善を行っています
- バグ報告や機能提案は [Issue](https://github.com/y-temp4/minwada/issues) からお願いします
