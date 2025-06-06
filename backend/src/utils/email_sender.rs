use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::email::{get_email_sender, EmailMessage};
use crate::error::AppError;
use crate::models::User;
use crate::utils::email_verification;

// メール検証用メール送信関数
pub async fn send_verification_email(
    user: &User,
    verification_token: &str,
) -> Result<(), AppError> {
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let verification_path =
        std::env::var("EMAIL_VERIFICATION_PATH").unwrap_or_else(|_| "/verify-email".to_string());

    let verification_url = format!(
        "{}{}/{}",
        frontend_url, verification_path, verification_token
    );

    let html_body = format!(
        r#"
        <h1>メールアドレスの確認</h1>
        <p>こんにちは、{}さん</p>
        <p>あなたのメールアドレスを確認するために、以下のリンクをクリックしてください：</p>
        <p><a href="{}">メールアドレスを確認する</a></p>
        <p>このリンクは24時間後に期限切れになります。</p>
        <p>このメールにお心当たりがない場合は、無視していただいて構いません。</p>
        "#,
        user.username, verification_url
    );

    let text_body = format!(
        r#"
        メールアドレスの確認
        
        こんにちは、{}さん
        
        あなたのメールアドレスを確認するために、以下のリンクをクリックしてください：
        
        {}
        
        このリンクは24時間後に期限切れになります。
        
        このメールにお心当たりがない場合は、無視していただいて構いません。
        "#,
        user.username, verification_url
    );

    let message = EmailMessage {
        to: user.email.clone(),
        subject: "メールアドレスの確認".to_string(),
        html_body,
        text_body: Some(text_body),
    };

    let email_sender = get_email_sender();
    email_sender
        .send_email(message)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

// ユーザー作成時のメール検証フロー
pub async fn start_verification_flow(
    user: &User,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<String, AppError> {
    let verification_token = email_verification::create_verification_token(user.id, tx).await?;

    // トランザクションがコミットされる前に非同期処理を行うとロールバック時に問題が生じる可能性があるため、
    // ここではトークンのみ生成しておき、呼び出し元でトランザクションコミット後にメール送信を行う

    Ok(verification_token)
}

// メール検証メールの再送信
pub async fn resend_verification_email(user_id: Uuid, pool: &PgPool) -> Result<(), AppError> {
    let mut tx = pool.begin().await.map_err(|e| AppError::Database(e))?;

    // ユーザー情報の取得
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT 
            id, username, email, display_name, avatar_url, 
            email_verified::bool as "email_verified!",
            email_verified_at,
            verification_token,
            verification_token_expires_at,
            created_at as "created_at!", updated_at as "updated_at!"
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::NotFound)?;

    // 既に検証済みの場合はエラー
    if user.email_verified_at.is_some() {
        return Err(AppError::BadRequest(
            "メールアドレスは既に検証済みです".to_string(),
        ));
    }

    // 新しいトークンの生成
    let verification_token =
        email_verification::create_verification_token(user.id, &mut tx).await?;

    // トランザクションのコミット
    tx.commit().await.map_err(|e| AppError::Database(e))?;

    // メール送信
    send_verification_email(&user, &verification_token).await?;

    Ok(())
}
