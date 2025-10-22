use chrono::{Duration, Utc};
use common::AppResult;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Token的Claims（声明）
/// 
/// 包含在JWT token中的用户信息和元数据。
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject - 用户ID（字符串格式的UUID）
    pub sub: String,
    /// Expiration Time - Token过期时间（Unix时间戳）
    pub exp: i64,
    /// Issued At - Token签发时间（Unix时间戳）
    pub iat: i64,
}

/// JWT服务接口
/// 
/// 提供JWT token的生成和验证功能。
pub trait JwtService: Send + Sync {
    /// 为用户生成JWT token
    /// 
    /// ## 参数
    /// - `user_id`: 用户UUID
    /// 
    /// ## 返回值
    /// 签名后的JWT token字符串
    fn generate_token(&self, user_id: Uuid) -> AppResult<String>;
    
    /// 验证并解析JWT token
    /// 
    /// ## 参数
    /// - `token`: JWT token字符串
    /// 
    /// ## 返回值
    /// 解析后的Claims，包含用户ID和过期时间
    /// 
    /// ## 错误
    /// - Token格式错误
    /// - Token已过期
    /// - 签名验证失败
    fn validate_token(&self, token: &str) -> AppResult<Claims>;
}

/// JWT服务的实现
#[derive(Clone)]
pub struct JwtServiceImpl {
    /// JWT签名密钥
    secret: String,
    /// Token有效期（秒）
    expiration: i64,
}

impl JwtServiceImpl {
    /// 创建JWT服务实例
    /// 
    /// ## 参数
    /// - `secret`: 签名密钥（生产环境使用强随机密钥）
    /// - `expiration`: Token有效期（秒，如86400=24小时）
    pub fn new(secret: String, expiration: i64) -> Self {
        Self { secret, expiration }
    }
}

impl JwtService for JwtServiceImpl {
    fn generate_token(&self, user_id: Uuid) -> AppResult<String> {
        let now = Utc::now();
        let exp = (now + Duration::seconds(self.expiration)).timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            exp,
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| common::AppError::AuthError(format!("Failed to generate token: {}", e)))?;

        Ok(token)
    }

    fn validate_token(&self, token: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| common::AppError::AuthError(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }
}

