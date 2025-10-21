/// 认证和安全模块
/// 
/// 提供JWT token生成/验证和密码加密功能。
/// 
/// ## 模块结构
/// 
/// - `jwt`: JWT token的生成和验证
/// - `password`: 密码的bcrypt加密和验证
/// 
/// ## 使用示例
/// 
/// ```rust
/// // 密码加密
/// let password_service = PasswordServiceImpl::new();
/// let hash = password_service.hash_password("password123")?;
/// 
/// // 密码验证
/// let is_valid = password_service.verify_password("password123", &hash)?;
/// 
/// // 生成JWT token
/// let jwt_service = JwtServiceImpl::new(secret, expiration);
/// let token = jwt_service.generate_token(user_id)?;
/// 
/// // 验证JWT token
/// let claims = jwt_service.validate_token(&token)?;
/// ```

pub mod jwt;
pub mod password;

pub use jwt::{JwtService, JwtServiceImpl, Claims};
pub use password::{PasswordService, PasswordServiceImpl};

