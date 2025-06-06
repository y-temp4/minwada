use crate::email::{get_email_sender, EmailMessage};
use crate::error::AppError;
use crate::models::User;

// パスワードリセット用メール送信関数
pub async fn send_password_reset_email(user: &User, reset_token: &str) -> Result<(), AppError> {
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let reset_path =
        std::env::var("PASSWORD_RESET_PATH").unwrap_or_else(|_| "/reset-password".to_string());

    let reset_url = format!("{}{}/{}", frontend_url, reset_path, reset_token);

    let html_body = format!(
        r#"
        <h1>パスワードリセットのリクエスト</h1>
        <p>こんにちは、{}さん</p>
        <p>パスワードリセットのリクエストを受け付けました。以下のリンクをクリックして新しいパスワードを設定してください：</p>
        <p><a href="{}">パスワードをリセットする</a></p>
        <p>このリンクは1時間後に期限切れになります。</p>
        <p>このリクエストにお心当たりがない場合は、このメールを無視していただいて構いません。アカウントは安全です。</p>
        "#,
        user.username, reset_url
    );

    let text_body = format!(
        r#"
        パスワードリセットのリクエスト
        
        こんにちは、{}さん
        
        パスワードリセットのリクエストを受け付けました。以下のリンクをクリックして新しいパスワードを設定してください：
        
        {}
        
        このリンクは1時間後に期限切れになります。
        
        このリクエストにお心当たりがない場合は、このメールを無視していただいて構いません。アカウントは安全です。
        "#,
        user.username, reset_url
    );

    let message = EmailMessage {
        to: user.email.clone(),
        subject: "パスワードリセットのリクエスト".to_string(),
        html_body,
        text_body: Some(text_body),
    };

    let email_sender = get_email_sender();
    email_sender
        .send_email(message)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}
