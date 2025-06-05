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

# フロントエンドの開発サーバーを起動（API生成ウォッチャーも並列実行）
frontend-dev:
    #!/usr/bin/env zsh
    function frontend_cleanup() {
        echo "🛑 フロントエンド終了中..."

        # Nodeプロセスも確実に終了させる
        if pkill -f "next" 2>/dev/null; then
            echo "✅ Nextプロセスを終了しました"
        fi
        # Orvalのプロセスは勝手に終了するので特に処理は不要
        # 少し待機して、プロセスが確実に終了するようにする
        sleep 0.5

        exit 0
    }

    trap frontend_cleanup INT TERM

    cd {{FRONTEND_DIR}}

    (
        npm run dev | while IFS= read -r line; do
            echo "$(printf '\033[34m[NEXT]\033[0m') 🚀 $line"
        done
    ) &

    (
        npm run generate-api:watch | while IFS= read -r line; do
            echo "$(printf '\033[36m[API_GEN]\033[0m') 🔄 $line"
        done
    ) &

    wait

# データベースをDocker Composeで起動（既に起動している場合はスキップ）
db-up:
    #!/usr/bin/env zsh
    cd {{BACKEND_DIR}} 
    if [ "$(docker compose ps -q 2>/dev/null | wc -l)" -gt 0 ]; then
        echo "✅ データベースはすでに実行中です。スキップします。"
    else
        echo "🔌 データベースを起動しています..."
        docker compose up -d
    fi

# データベースをDocker Composeで停止
db-stop:
    cd {{BACKEND_DIR}} && docker compose stop

# すべてのサービスを起動（バックエンド、フロントエンド、DB）
dev: db-up
    #!/usr/bin/env zsh
    # バックグラウンドプロセスを終了するための関数
    function dev_cleanup() {
        echo "🛑 開発環境終了中..."

        # 現在のディレクトリを保存
        local CURRENT_DIR=$(pwd)

        # まずcargo-watchを終了させる
        if pkill -f "cargo-watch" 2>/dev/null; then
            echo "✅ Cargo Watchプロセスを終了しました"
        fi
        # すべてのreddit-backendプロセスがあれば終了させる
        # target/debug/reddit-backendの形式でプロセスを検索して終了
        if pkill -f "target/debug/reddit-backend" 2>/dev/null; then
            echo "✅ Rustバックエンドプロセスを終了しました"
        fi
        # 念のためプロセス名でも検索して終了
        if pkill -f "reddit-backend" 2>/dev/null; then
            echo "✅ 追加のRustバックエンドプロセスを終了しました"
        fi
        # Next.jsを終了させる
        if pkill -f "next" 2>/dev/null; then
            echo "✅ Nextプロセスを終了しました"
        fi
        # Docker Composeを停止する
        local ABSOLUTE_BACKEND_DIR="$(pwd)/{{BACKEND_DIR}}"
        if [ -d "$ABSOLUTE_BACKEND_DIR" ]; then
            cd "$ABSOLUTE_BACKEND_DIR" && docker compose stop
            echo "✅ Docker Composeのコンテナを停止しました"
        else
            echo "⚠️ バックエンドディレクトリが見つからないため、Docker Composeを停止できませんでした"
        fi
        # 元のディレクトリに戻る
        cd "$CURRENT_DIR"

        exit 0
    }

    # シグナルハンドラを設定 (EXITを除外して二重実行を防止)
    trap dev_cleanup INT TERM

    # バックエンドとフロントエンドを並列に起動し、ログに色付きプレフィックスを追加
    (
        cd {{BACKEND_DIR}} && 
        cargo watch -x run | while IFS= read -r line; do
            echo "$(printf '\033[32m[BACKEND]\033[0m') 🦀 $line"
        done
    ) &

    (
        cd {{FRONTEND_DIR}} && 
        npm run dev | while IFS= read -r line; do
            echo "$(printf '\033[34m[NEXT]\033[0m') 🚀 $line"
        done
    ) &

    (
        cd {{FRONTEND_DIR}} && 
        npm run generate-api:watch | while IFS= read -r line; do
            echo "$(printf '\033[36m[API_GEN]\033[0m') 🔄 $line"
        done
    ) &

    # すべてのバックグラウンドプロセスが終了するまで待機
    wait


# フロントエンドのインストール
frontend-install:
    cd {{FRONTEND_DIR}} && npm ci

# バックエンドのビルド
backend-build:
    cd {{BACKEND_DIR}} && cargo build


# プロジェクト全体のセットアップ
setup: frontend-install backend-build db-up
    @echo "🎉 セットアップが完了しました。"
