use std::fmt;

/// 应用程序统一结果类型
/// 
/// 使用该类型可以简化错误处理，所有业务逻辑都返回`AppResult<T>`
/// 
/// ## 示例
/// ```rust
/// async fn get_user(id: Uuid) -> AppResult<User> {
///     repository.find_by_id(id)
///         .await?
///         .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))
/// }
/// ```
pub type AppResult<T> = Result<T, AppError>;

/// 应用程序统一错误类型
/// 
/// 该枚举定义了应用中可能出现的所有业务错误类型。
/// 通过统一的错误类型，可以在不同层之间传递错误，并在API层统一处理。
/// 
/// ## 错误分类
/// 
/// ### DatabaseError - 数据库错误
/// - SQL执行失败
/// - 连接池耗尽
/// - 事务冲突
/// 
/// ### NotFound - 资源不存在
/// - 用户不存在
/// - 模板不存在
/// - 清单不存在
/// 
/// ### ValidationError - 数据验证失败
/// - 输入格式错误
/// - 业务规则违反
/// - 参数缺失
/// 
/// ### AuthError - 认证/授权失败
/// - 密码错误
/// - Token无效/过期
/// - 权限不足
/// 
/// ### InternalError - 内部错误
/// - 未预期的错误
/// - 系统配置错误
/// - 第三方服务错误
#[derive(Debug)]
pub enum AppError {
    /// 数据库操作错误
    /// 
    /// 包括：SQL错误、连接错误、事务错误等
    DatabaseError(String),
    
    /// 资源未找到错误
    /// 
    /// 应返回HTTP 404，用于查询不存在的资源
    NotFound(String),
    
    /// 数据验证错误
    /// 
    /// 应返回HTTP 400，用于客户端提交的数据不符合要求
    ValidationError(String),
    
    /// 认证/授权错误
    /// 
    /// 应返回HTTP 401（未认证）或403（无权限）
    AuthError(String),
    
    /// 内部服务器错误
    /// 
    /// 应返回HTTP 500，用于未预期的错误
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            AppError::NotFound(msg) => write!(f, "未找到: {}", msg),
            AppError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            AppError::AuthError(msg) => write!(f, "认证错误: {}", msg),
            AppError::InternalError(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// 自动将anyhow错误转换为AppError
/// 
/// 用于处理通用错误场景
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

/// 自动将SeaORM DbErr转换为AppError
/// 
/// 这允许在Repository层使用`?`操作符直接传播SeaORM错误
impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

/// 自动将serde_json错误转换为AppError
/// 
/// 用于处理JSON序列化/反序列化错误
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::InternalError(format!("JSON error: {}", err))
    }
}

