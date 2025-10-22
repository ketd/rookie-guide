use axum::{extract::State, http::StatusCode, Json};
use models::{RegisterDto, LoginDto, AuthResponse};
use common::ApiResponse;
use crate::state::AppState;

/// 用户注册处理器
/// 
/// ## 端点
/// POST /api/auth/register
/// 
/// ## 请求体
/// ```json
/// {
///   "phone": "13800138000",      // 手机号（可选，与email二选一）
///   "email": "user@example.com", // 邮箱（可选，与phone二选一）
///   "password": "password123",   // 密码（至少6位）
///   "nickname": "张三"            // 昵称（1-50字符）
/// }
/// ```
/// 
/// ## 响应
/// - 200 OK: 注册成功，返回用户信息和JWT token
/// - 400 Bad Request: 验证失败或用户已存在
/// 
/// ## 业务逻辑
/// 1. 验证输入数据（手机号/邮箱格式、密码长度等）
/// 2. 检查用户是否已存在
/// 3. 使用bcrypt加密密码
/// 4. 创建用户记录
/// 5. 生成JWT token
/// 6. 返回用户信息和token
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterDto,
    responses(
        (status = 200, description = "注册成功", body = ApiResponse<AuthResponse>),
        (status = 400, description = "验证失败或用户已存在")
    ),
    tag = "认证"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDto>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // 从依赖注入容器获取用户服务
    let user_service = &state.module.user_service;
    
    // 调用业务逻辑层处理注册
    let response = user_service
        .register(dto)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(response))
}

/// 用户登录处理器
/// 
/// ## 端点
/// POST /api/auth/login
/// 
/// ## 请求体
/// ```json
/// {
///   "phone": "13800138000",      // 手机号（与email二选一）
///   "email": "user@example.com", // 邮箱（与phone二选一）
///   "password": "password123"    // 密码
/// }
/// ```
/// 
/// ## 响应
/// - 200 OK: 登录成功，返回用户信息和JWT token
/// - 401 Unauthorized: 用户名或密码错误
/// 
/// ## 业务逻辑
/// 1. 根据手机号或邮箱查找用户
/// 2. 验证密码（bcrypt.verify）
/// 3. 生成JWT token（包含用户ID和过期时间）
/// 4. 返回用户信息和token
/// 
/// ## 安全性
/// - 密码使用bcrypt验证，不会明文存储
/// - JWT token设置过期时间（默认24小时）
/// - 登录失败不泄露具体原因（用户不存在 vs 密码错误）
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginDto,
    responses(
        (status = 200, description = "登录成功", body = ApiResponse<AuthResponse>),
        (status = 401, description = "用户名或密码错误")
    ),
    tag = "认证"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // 从依赖注入容器获取用户服务
    let user_service = &state.module.user_service;
    
    // 调用业务逻辑层处理登录
    // 如果验证失败，返回401 Unauthorized
    let response = user_service
        .login(dto)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    Ok(Json(response))
}

