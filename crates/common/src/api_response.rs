/// OpenAPI 统一响应结构
/// 
/// 为所有API返回统一的JSON格式，包含成功状态、消息、数据和时间戳

use serde::Serialize;
use utoipa::ToSchema;
use axum::{
    Json,
    response::{IntoResponse, Response},
    http::StatusCode,
};

/// 统一 API 响应结构
/// 
/// ## 字段说明
/// - `success`: 请求是否成功
/// - `message`: 提示信息（成功消息或错误描述）
/// - `data`: 响应数据（成功时包含，失败时为null）
/// - `timestamp`: 响应时间戳（毫秒）
/// 
/// ## 成功响应示例
/// ```json
/// {
///   "success": true,
///   "message": "创建成功",
///   "data": { "id": "...", "name": "..." },
///   "timestamp": 1730000000000
/// }
/// ```
/// 
/// ## 失败响应示例
/// ```json
/// {
///   "success": false,
///   "message": "用户不存在",
///   "data": null,
///   "timestamp": 1730000000000
/// }
/// ```
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    /// 请求是否成功
    pub success: bool,
    
    /// 提示信息
    pub message: String,
    
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    
    /// 响应时间戳（毫秒）
    pub timestamp: i64,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应
    /// 
    /// ## 参数
    /// - `data`: 响应数据
    /// - `message`: 成功消息
    /// 
    /// ## 示例
    /// ```rust
    /// let response = ApiResponse::success(user, "获取成功");
    /// Ok(response)
    /// ```
    pub fn success(data: T, message: impl Into<String>) -> Json<Self> {
        Json(Self {
            success: true,
            message: message.into(),
            data: Some(data),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }
    
    /// 创建成功响应（无数据）
    /// 
    /// 用于删除、更新等不需要返回数据的操作
    pub fn success_no_data(message: impl Into<String>) -> Json<Self> 
    where
        T: Default,
    {
        Json(Self {
            success: true,
            message: message.into(),
            data: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }
    
    /// 创建错误响应
    pub fn error(message: impl Into<String>) -> Json<Self> {
        Json(Self {
            success: false,
            message: message.into(),
            data: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }
}

/// API错误类型（可转换为HTTP响应）
/// 
/// 自动转换为带有适当HTTP状态码的ApiResponse
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// 400 - 请求参数错误
    #[error("请求参数错误: {0}")]
    BadRequest(String),
    
    /// 401 - 未认证
    #[error("未认证: {0}")]
    Unauthorized(String),
    
    /// 403 - 无权限
    #[error("无权限: {0}")]
    Forbidden(String),
    
    /// 404 - 资源不存在
    #[error("资源不存在: {0}")]
    NotFound(String),
    
    /// 500 - 服务器内部错误
    #[error("服务器错误: {0}")]
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        let response: ApiResponse<()> = ApiResponse {
            success: false,
            message,
            data: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        
        (status, Json(response)).into_response()
    }
}

/// 将AppError转换为ApiError
impl From<crate::AppError> for ApiError {
    fn from(err: crate::AppError) -> Self {
        match err {
            crate::AppError::NotFound(msg) => ApiError::NotFound(msg),
            crate::AppError::ValidationError(msg) => ApiError::BadRequest(msg),
            crate::AppError::AuthError(msg) => ApiError::Unauthorized(msg),
            crate::AppError::DatabaseError(msg) => ApiError::InternalError(format!("数据库错误: {}", msg)),
            crate::AppError::InternalError(msg) => ApiError::InternalError(msg),
        }
    }
}

