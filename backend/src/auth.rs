// JWT and authentication utilities

pub mod jwt {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

    use crate::{error::AppError, models::auth::Claims};

    pub fn create_jwt_token(
        user_id: &str,
        username: &str,
        email: &str,
        secret: &str,
        expires_in_minutes: i64,
    ) -> Result<String, AppError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(expires_in_minutes))
            .expect("Valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            email: email.to_string(),
            exp: expiration,
            iat: Utc::now().timestamp() as usize,
            iss: "wadai-us".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(AppError::Jwt)
    }

    pub fn verify_jwt_token(token: &str, secret: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(AppError::Jwt)
    }
}

pub mod password {
    use argon2::{
        password_hash::{
            rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        },
        Argon2,
    };

    use crate::error::AppError;

    pub fn hash_password(password: &str) -> Result<(String, String), AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

        Ok((password_hash.to_string(), salt.to_string()))
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash)?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
