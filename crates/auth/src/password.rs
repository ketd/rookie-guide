use common::AppResult;

/// 密码服务接口
/// 
/// 提供密码的加密和验证功能，使用bcrypt算法。
/// 
/// ## 安全性
/// 
/// - 使用bcrypt算法（自带salt）
/// - 成本因子：DEFAULT_COST（当前为12）
/// - 不可逆加密，无法从哈希值还原密码
/// - 每次加密同一密码会产生不同的哈希值（盐值随机）
pub trait PasswordService: Send + Sync {
    /// 加密密码
    /// 
    /// ## 参数
    /// - `password`: 明文密码
    /// 
    /// ## 返回值
    /// bcrypt哈希字符串（包含算法、成本因子、盐值和哈希值）
    /// 
    /// ## 示例
    /// ```
    /// 输入：    "password123"
    /// 输出：    "$2b$12$KIXxLx.../hash..."
    /// ```
    fn hash_password(&self, password: &str) -> AppResult<String>;
    
    /// 验证密码
    /// 
    /// ## 参数
    /// - `password`: 用户输入的明文密码
    /// - `hash`: 存储的bcrypt哈希值
    /// 
    /// ## 返回值
    /// - `true`: 密码正确
    /// - `false`: 密码错误
    fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool>;
}

/// 密码服务的实现
#[derive(Clone)]
pub struct PasswordServiceImpl;

impl PasswordServiceImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PasswordServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordService for PasswordServiceImpl {
    /// 使用bcrypt加密密码
    /// 
    /// 成本因子使用bcrypt::DEFAULT_COST（当前为12）
    fn hash_password(&self, password: &str) -> AppResult<String> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| common::AppError::InternalError(format!("密码加密失败: {}", e)))
    }

    /// 验证密码是否匹配
    /// 
    /// bcrypt会自动从哈希值中提取盐值进行验证
    fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        bcrypt::verify(password, hash)
            .map_err(|e| common::AppError::AuthError(format!("密码验证失败: {}", e)))
    }
}

