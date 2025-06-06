FROM rust:1.87-alpine3.22 AS builder

WORKDIR /app

# 実際のソースコードをコピー
COPY . .

ARG CORS_ORIGIN \
    DATABASE_URL

# アプリケーションをビルド
RUN cargo build --release

# 実行ステージ
FROM builder AS runner

WORKDIR /app

# ビルドステージからバイナリをコピー
COPY --from=builder /app/target/release/reddit-backend /app/

# 非rootユーザーに切り替え
USER rust

EXPOSE 8000

# アプリケーションの実行
CMD ["/app/reddit-backend"]
