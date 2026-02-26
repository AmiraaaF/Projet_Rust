use crate::models::Claims;
use crate::errors::{AppError, AppResult};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

pub struct AuthService {
    jwt_secret: String,
    jwt_expiration: i64,
}

impl AuthService {
    pub fn new(jwt_secret: String, jwt_expiration: i64) -> Self {
        Self {
            jwt_secret,
            jwt_expiration,
        }
    }

    pub fn generate_token(&self, user_id: Uuid, email: String, role: String) -> AppResult<String> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            email,
            role,
            iat: now,
            exp: now + self.jwt_expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|_| AppError::InternalError("Failed to generate token".to_string()))
    }

    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::InvalidToken("Invalid or expired token".to_string()))
    }
}

pub fn hash_password(password: &str) -> AppResult<String> {
    bcrypt::hash(password, 12)
        .map_err(|_| AppError::InternalError("Failed to hash password".to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    bcrypt::verify(password, hash)
        .map_err(|_| AppError::InternalError("Failed to verify password".to_string()))
}
