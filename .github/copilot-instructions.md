# バックエンド

- ルーティング時に動的なパスを生成する場合は、既存のコードを参考にしてください
- API エンドポイントを作成・変更した場合は、main.rs にエンドポイントを登録・修正してください

## テスト

- handlers のテストは基本的に sqlx のテスト機能を利用してください

# フロントエンド

## デザイン

- 色を参照する際は基本的に globals.css に ある変数を利用してください

## API コール

- API クライアントは、常に generated ディレクトリ配下に生成されたものを使用してください
- API の呼び出し中は、TanStack Query の isPending もしくは React Hook Form の isSubmitting を利用して、ローディング状態を管理してください
- API の呼び出し後にページ遷移を伴う場合などは、isPending もしくは isSubmitting だけでは不十分です
  - そのため、TanStack Query の isSuccess もしくは React Hook Form の isSubmitSuccessful を利用して、API の呼び出しが成功したことを確認してください

# 共通

- 基本的に開発時は npm run generate-api:watch を実行しているため、バックエンドの変更後にフロントエンドのコードを生成する必要はありません
  - 生成がうまくいっていない場合は、npm run generate-api を実行してください
