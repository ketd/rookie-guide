use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use auth::{JwtService, JwtServiceImpl};
use common::AppConfig;
use uuid::Uuid;

/// 当前登录用户信息
/// 
/// 该结构体通过JWT认证中间件自动提取，包含当前请求的用户ID。
/// 在需要认证的handler中，可以直接将其作为参数使用。
/// 
/// ## 使用示例
/// ```rust
/// async fn protected_handler(
///     current_user: CurrentUser,  // 自动提取当前用户
/// ) -> impl IntoResponse {
///     format!("Hello, user {}", current_user.user_id)
/// }
/// ```
pub struct CurrentUser {
    /// 当前登录用户的UUID
    pub user_id: Uuid,
}

/// JWT认证中间件
/// 
/// 实现了Axum的`FromRequestParts` trait，自动从请求中提取并验证JWT token。
/// 
/// ## 认证流程：
/// 1. 从请求头中提取 `Authorization: Bearer <token>`
/// 2. 验证token格式是否正确
/// 3. 使用JWT服务验证token签名和有效期
/// 4. 从token的claims中提取用户ID
/// 5. 返回`CurrentUser`实例
/// 
/// ## 错误处理：
/// - 401 Unauthorized: token缺失、格式错误、验证失败、已过期
/// - 500 Internal Server Error: 配置加载失败
#[async_trait::async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // ==================== 1. 提取Authorization头 ====================
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED, 
                "缺少Authorization请求头".to_string()
            ))?;

        // ==================== 2. 验证Bearer格式 ====================
        // 标准格式: "Authorization: Bearer <token>"
        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED, 
                "Authorization头格式错误，应为: Bearer <token>".to_string()
            ));
        }

        // 移除"Bearer "前缀，提取token
        let token = &auth_header[7..];

        // ==================== 3. 加载配置并验证token ====================
        // 加载JWT密钥配置
        let config = AppConfig::from_env()
            .map_err(|_| (
                StatusCode::INTERNAL_SERVER_ERROR, 
                "服务器配置加载失败".to_string()
            ))?;
        
        // 创建JWT服务实例
        let jwt_service = JwtServiceImpl::new(
            config.jwt.secret, 
            config.jwt.expiration
        );
        
        // 验证token并提取claims
        let claims = jwt_service
            .validate_token(token)
            .map_err(|e| (
                StatusCode::UNAUTHORIZED, 
                format!("Token验证失败: {}", e)
            ))?;

        // ==================== 4. 解析用户ID ====================
        // 从claims中提取用户ID（sub字段）
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| (
                StatusCode::UNAUTHORIZED, 
                "Token中的用户ID格式无效".to_string()
            ))?;

        // 返回当前用户信息
        Ok(CurrentUser { user_id })
    }
}

