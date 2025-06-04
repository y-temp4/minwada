# reddit-sampleプロジェクトのjustfile
# 使用方法: just <コマンド>

# ディレクトリ定義
BACKEND_DIR := 'backend'
FRONTEND_DIR := 'frontend'

# デフォルトのコマンドリスト表示
default:
    @just --list

# バックエンドの開発サーバーを起動（cargo watchを使用）
backend-dev:
    cd {{BACKEND_DIR}} && cargo watch -x run

# フロントエンドの開発サーバーを起動
frontend-dev:
    cd {{FRONTEND_DIR}} && npm run dev

# データベースをDocker Composeで起動
db-up:
    cd {{BACKEND_DIR}} && docker compose up -d

# データベースをDocker Composeで停止
db-down:
    cd {{BACKEND_DIR}} && docker compose down

# すべてのサービスを起動（バックエンド、フロントエンド、DB）
dev:
    just db-up
    just dev-parallel


# フロントエンドのインストール
frontend-install:
    cd {{FRONTEND_DIR}} && npm install

# バックエンドのビルド
backend-build:
    cd {{BACKEND_DIR}} && cargo build


# プロジェクト全体のセットアップ
setup: frontend-install backend-build db-up
    @echo "セットアップが完了しました。"

# バックエンドとフロントエンドを並列で実行
dev-parallel:
    #!/usr/bin/env zsh
    # バックグラウンドプロセスを終了するための関数
    function cleanup() {
        echo "\n終了中..."
        # 明示的にジョブ番号を指定して終了
        jobs | while read -r job; do
            job_id=$(echo $job | grep -o '\[[0-9]*\]' | tr -d '[]')
            if [[ -n "$job_id" ]]; then
                kill %$job_id 2>/dev/null || true
            fi
        done
        exit 0
    }
    
    # シグナルハンドラを設定
    trap cleanup INT TERM EXIT
    
    # バックエンドとフロントエンドを並列に起動し、ログに色付きプレフィックスを追加
    # macOS用のログ出力
    (
        cd {{BACKEND_DIR}} && 
        cargo watch -x run | while IFS= read -r line; do
            echo "$(printf '\033[32m[BACKEND]\033[0m') $line"
        done
    ) &
    
    (
        cd {{FRONTEND_DIR}} && 
        npm run dev | while IFS= read -r line; do
            echo "$(printf '\033[34m[FRONTEND]\033[0m') $line"
        done
    ) &
    
    # すべてのバックグラウンドプロセスが終了するまで待機
    wait
