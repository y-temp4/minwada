# wadai-us Frontend

Next.js 15 (App Router) + shadcn/ui で構築された Reddit 風サービスのフロントエンドです。

## 技術スタック

- **Framework**: Next.js 15+ (App Router)
- **UI**: shadcn/ui + Tailwind CSS
- **状態管理**: TanStack Query
- **API Client**: Orval (OpenAPI から自動生成)
- **フォーム**: React Hook Form + Zod
- **認証**: JWT (localStorage)
- **通知**: Sonner

## 機能

- レスポンシブデザイン
- スレッド一覧・作成
- コメント機能（木構造）
- ユーザー認証
- リアルタイム更新
- 型安全な API クライアント

## 開発環境セットアップ

### 1. 前提条件

- Node.js (`.tool-versions` で指定されたバージョン)
- npm
- バックエンド API（Rust）が起動していること

### 2. 依存関係インストール

```bash
npm ci
```

### 3. 環境変数設定

```bash
# 環境変数ファイルをコピー
cp env.example .env.local

# .env.localファイルを編集
```

### 4. API クライアント生成

```bash
# バックエンドが起動している必要があります
npm run generate-api
```

### 5. 開発サーバー起動

```bash
npm run dev
```

アプリケーションは http://localhost:3000 で起動します。

## 利用可能なスクリプト

```bash
# 開発サーバー起動
npm run dev

# 本番ビルド
npm run build

# 本番サーバー起動
npm run start

# リンター実行
npm run lint

# APIクライアント生成
npm run generate-api

# APIクライアント生成（ウォッチモード）
npm run generate-api:watch
```

## プロジェクト構造

```
frontend/
├── src/
│   ├── app/                    # Next.js App Router
│   │   ├── layout.tsx         # ルートレイアウト
│   │   ├── page.tsx           # ホームページ
│   │   ├── login/             # ログインページ
│   │   └── register/          # 登録ページ
│   ├── components/            # Reactコンポーネント
│   │   ├── ui/               # shadcn/uiコンポーネント
│   │   ├── thread-list.tsx   # スレッド一覧
│   │   └── create-thread-form.tsx # スレッド作成フォーム
│   ├── lib/                  # ユーティリティ
│   │   ├── utils.ts          # 共通ユーティリティ
│   │   └── api-client.ts     # APIクライアント設定
│   ├── providers/            # Reactプロバイダー
│   │   └── query-client-provider.tsx
│   └── generated/            # Orval生成ファイル
│       ├── api.ts           # APIクライアント
│       └── schemas/         # 型定義
├── orval.config.ts          # Orval設定
├── components.json          # shadcn/ui設定
├── tailwind.config.ts       # Tailwind設定
└── package.json            # 依存関係
```

## API 統合

### Orval による自動生成

バックエンドの OpenAPI 仕様から型安全な API クライアントを自動生成：

```bash
# APIクライアント生成
npm run generate-api
```

### 使用例

```typescript
import { useGetThreads, useCreateThread } from "@/generated/api";

// スレッド一覧取得
const { data: threads, isLoading } = useGetThreads();

// スレッド作成
const createThread = useCreateThread({
  onSuccess: () => {
    // 成功時の処理
  },
});
```

## 認証

JWT トークンを localStorage で管理し、API リクエストに自動付与：

```typescript
// ログイン後
localStorage.setItem("access_token", token);

// 自動的にAuthorizationヘッダーに付与
// Authorization: Bearer <token>
```

## スタイリング

### shadcn/ui

モダンで美しい UI コンポーネント：

```bash
# 新しいコンポーネント追加
npx shadcn@latest add <component-name>
```

### Tailwind CSS

ユーティリティファーストの CSS：

```typescript
<div className="bg-white border rounded-lg p-4 shadow-sm">Content</div>
```

## 開発ワークフロー

1. バックエンドで API 仕様変更
2. `npm run generate-api`でクライアント再生成
3. 型安全な API クライアントを使用
4. UI コンポーネントを実装

## 注意事項

- バックエンド API が起動している必要があります
- API クライアント生成前に OpenAPI 仕様が利用可能である必要があります
- 本プロジェクトは学習・デモ用途です
