use axum::{extract::State, http::StatusCode, Json};
use models::{UserProfile, UpdateProfileDto};
use common::ApiResponse;
use crate::{middleware::CurrentUser, state::AppState};

/// 获取当前登录用户信息
/// 
/// ## 端点
/// GET /api/users/me
/// 
/// ## 认证
/// 需要JWT token（通过Authorization头）
/// 
/// ## 响应
/// - 200 OK: 返回用户公开资料
/// - 401 Unauthorized: Token无效或未提供
/// - 404 Not Found: 用户不存在（理论上不会发生，因为token验证时已确认）
/// 
/// ## 响应示例
/// ```json
/// {
///   "id": "uuid",
///   "nickname": "张三",
///   "avatar_url": "https://example.com/avatar.jpg",
///   "home_city": "CN-BJ"
/// }
/// ```
/// 
/// ## 业务逻辑
/// 1. JWT中间件自动验证token并提取用户ID
/// 2. 根据用户ID查询数据库
/// 3. 返回用户公开资料（不包含密码、手机号等敏感信息）
/// 
/// ## 安全性
/// - 只返回公开信息（UserProfile），不包含password_hash
/// - Token验证在中间件层完成
/// - 用户只能查看自己的信息（通过JWT中的user_id）
/// 
/// ## 使用场景
/// - 前端应用初始化时获取当前用户信息
/// - 刷新用户资料
/// - 显示用户头像和昵称
#[utoipa::path(
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<UserProfile>),
        (status = 401, description = "未认证"),
        (status = 404, description = "用户不存在")
    ),
    security(("bearer_auth" = [])),
    tag = "用户"
)]
pub async fn get_current_user(
    State(state): State<AppState>,
    current_user: CurrentUser,  // JWT认证中间件自动注入
) -> Result<Json<UserProfile>, (StatusCode, String)> {
    // 从依赖注入容器获取用户服务
    let user_service = &state.module.user_service;
    
    // 查询用户信息
    let profile = user_service
        .get_user(current_user.user_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(profile))
}

/// 更新当前用户资料
/// 
/// ## 端点
/// PUT /api/users/me
/// 
/// ## 认证
/// 需要JWT token
/// 
/// ## 请求体
/// ```json
/// {
///   "nickname": "新昵称",           // 可选，1-50字符
///   "avatar_url": "https://...",   // 可选，头像URL
///   "home_city": "CN-SH"           // 可选，常驻城市代码
/// }
/// ```
/// 
/// 所有字段都是可选的，只更新提供的字段。
/// 
/// ## 响应
/// - 200 OK: 更新成功，返回更新后的用户资料
/// - 400 Bad Request: 验证失败（如昵称过长）
/// - 401 Unauthorized: Token无效
/// 
/// ## 业务逻辑
/// 1. 验证输入数据（昵称长度、URL格式等）
/// 2. 更新用户记录（只更新提供的字段）
/// 3. 返回更新后的用户资料
/// 
/// ## 验证规则
/// - `nickname`: 1-50字符
/// - `avatar_url`: 任意URL字符串
/// - `home_city`: 城市代码（如"CN-BJ"、"CN-SH"等）
/// 
/// ## 使用场景
/// - 用户修改昵称
/// - 用户上传/更换头像
/// - 用户设置常驻城市（用于个性化模板推荐）
/// 
/// ## 注意事项
/// - V0.0.1版本不支持修改手机号和邮箱
/// - 修改常驻城市后，模板推荐会优先显示该城市的本地化模板
#[utoipa::path(
    put,
    path = "/api/users/me",
    request_body = UpdateProfileDto,
    responses(
        (status = 200, description = "更新成功", body = ApiResponse<UserProfile>),
        (status = 400, description = "验证失败"),
        (status = 401, description = "未认证")
    ),
    security(("bearer_auth" = [])),
    tag = "用户"
)]
pub async fn update_profile(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(dto): Json<UpdateProfileDto>,
) -> Result<Json<UserProfile>, (StatusCode, String)> {
    // 从依赖注入容器获取用户服务
    let user_service = &state.module.user_service;
    
    // 更新用户资料
    let profile = user_service
        .update_profile(current_user.user_id, dto)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(profile))
}

