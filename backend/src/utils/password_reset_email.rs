use sqlx::PgPool;

use crate::email::{get_email_sender, EmailMessage};
use crate::error::AppError;
use crate::models::User;
use crate::utils::password_reset;

// パスワードリセット用メール送信関数
pub async fn send_password_reset_email(user: &User, reset_token: &str) -> Result<(), AppError> {
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let reset_path =
        std::env::var("PASSWORD_RESET_PATH").unwrap_or_else(|_| "/reset-password".to_string());

    let reset_url = format!("{}{}/{}", frontend_url, reset_path, reset_token);

    let html_body = format!(
        r#"
        <h1>パスワードリセットのご案内</h1>
        <p>こんにちは、{}さん</p>
        <p>パスワードをリセットするために、以下のリンクをクリックしてください：</p>
        <p><a href="{}">パスワードをリセットする</a></p>
        <p>このリンクは1時間後に期限切れになります。</p>
        <p>このメールにお心当たりがない場合は、無視していただいて構いません。</p>
        "#,
        user.username, reset_url
    );

    let text_body = format!(
        r#"
        パスワードリセットのご案内
        
        こんにちは、{}さん
        
        パスワードをリセットするために、以下のリンクをクリックしてください：
        
        {}
        
        このリンクは1時間後に期限切れになります。
        
        このメールにお心当たりがない場合は、無視していただいて構いません。
        "#,
        user.username, reset_url
    );

    let message = EmailMessage {
        to: user.email.clone(),
        subject: "パスワードリセットのご案内".to_string(),
        html_body,
        text_body: Some(text_body),
    };

    let email_sender = get_email_sender();
    email_sender
        .send_email(message)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

// パスワードリセットフロー開始
pub async fn start_password_reset_flow(email: &str, pool: &PgPool) -> Result<(), AppError> {
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
            password_reset_token,
            password_reset_token_expires_at,
            created_at as "created_at!", updated_at as "updated_at!"
        FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e))?;

    match user {
        Some(user) => {
            // トークンの生成
            let reset_token = password_reset::create_reset_token(user.id, &mut tx).await?;

            // トランザクションのコミット
            tx.commit().await.map_err(|e| AppError::Database(e))?;

            // メール送信
            send_password_reset_email(&user, &reset_token).await?;

            Ok(())
        }
        None => {
            // セキュリティ上の理由から、ユーザーが存在しない場合でも
            // 成功レスポンスを返す（ユーザー列挙攻撃の防止）
            tx.commit().await.map_err(|e| AppError::Database(e))?;
            Ok(())
        }
    }
}
