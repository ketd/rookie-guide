use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

/// 用户模型（数据库实体）
/// 
/// 对应数据库表: `users`
/// 
/// ## 字段说明
/// - `id`: 用户唯一标识（UUID）
/// - `phone`: 手机号（可选，用于登录）
/// - `email`: 邮箱（可选，用于登录）
/// - `password_hash`: 密码哈希（bcrypt加密，永不返回给客户端）
/// - `nickname`: 用户昵称（显示名称）
/// - `avatar_url`: 头像URL（可选）
/// - `home_city`: 常驻城市（如"CN-BJ"，用于个性化推荐）
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
/// 
/// ## 安全性
/// - 密码使用bcrypt加密存储，成本因子为DEFAULT_COST
/// - 手机号和邮箱至少需要提供一个（数据库约束）
/// - 手机号和邮箱都有唯一索引，防止重复注册
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// 用户UUID（主键）
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    
    /// 手机号（11位，可选）
    pub phone: Option<String>,
    
    /// 邮箱地址（可选）
    pub email: Option<String>,
    
    /// 密码哈希值（bcrypt加密，不可逆）
    #[serde(skip_serializing)]  // 序列化时跳过此字段
    pub password_hash: String,
    
    /// 用户昵称
    pub nickname: String,
    
    /// 头像URL
    pub avatar_url: Option<String>,
    
    /// 常驻城市代码（如"CN-BJ"表示北京）
    /// 用于根据用户位置推荐相关模板
    pub home_city: Option<String>,
    
    /// 账户创建时间
    pub created_at: DateTime<Utc>,
    
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::template::Entity")]
    Templates,
    #[sea_orm(has_many = "super::user_checklist::Entity")]
    Checklists,
}

impl Related<super::template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Templates.def()
    }
}

impl Related<super::user_checklist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Checklists.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// 用户公开资料（安全的用户信息）
/// 
/// 该结构体用于API响应，不包含敏感信息（如密码哈希、手机号、邮箱）
/// 
/// ## 使用场景
/// - 返回当前登录用户信息
/// - 显示其他用户的公开资料
/// - 评论、清单等关联的用户信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserProfile {
    /// 用户ID
    pub id: Uuid,
    
    /// 用户昵称
    pub nickname: String,
    
    /// 头像URL
    pub avatar_url: Option<String>,
    
    /// 常驻城市
    pub home_city: Option<String>,
}

impl From<Model> for UserProfile {
    /// 从User转换为UserProfile，过滤掉敏感信息
    fn from(user: Model) -> Self {
        UserProfile {
            id: user.id,
            nickname: user.nickname,
            avatar_url: user.avatar_url,
            home_city: user.home_city,
        }
    }
}

/// 用户注册数据传输对象（DTO）
/// 
/// ## 验证规则
/// - `phone`: 11位手机号（与email二选一）
/// - `email`: 有效的邮箱格式（与phone二选一）
/// - `password`: 6-100个字符
/// - `nickname`: 1-50个字符
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterDto {
    /// 手机号（11位）
    #[validate(length(min = 11, max = 11))]
    pub phone: Option<String>,
    
    /// 邮箱地址
    #[validate(email)]
    pub email: Option<String>,
    
    /// 密码（明文，仅用于传输，存储时会加密）
    #[validate(length(min = 6, max = 100))]
    pub password: String,
    
    /// 用户昵称
    #[validate(length(min = 1, max = 50))]
    pub nickname: String,
}

/// 用户登录数据传输对象（DTO）
/// 
/// ## 验证规则
/// - `phone` 或 `email`: 至少提供一个
/// - `password`: 至少6个字符
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginDto {
    /// 手机号（与email二选一）
    pub phone: Option<String>,
    
    /// 邮箱（与phone二选一）
    pub email: Option<String>,
    
    /// 密码
    #[validate(length(min = 6))]
    pub password: String,
}

/// 更新用户资料数据传输对象（DTO）
/// 
/// 所有字段都是可选的，只更新提供的字段
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileDto {
    /// 新昵称
    #[validate(length(min = 1, max = 50))]
    pub nickname: Option<String>,
    
    /// 新头像URL
    pub avatar_url: Option<String>,
    
    /// 新常驻城市
    pub home_city: Option<String>,
}

/// 认证响应（注册/登录成功后的响应）
/// 
/// 包含用户信息和JWT token
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    /// 用户公开资料
    pub user: UserProfile,
    
    /// JWT访问令牌
    /// 
    /// 客户端应将其存储并在后续请求中通过
    /// `Authorization: Bearer <token>` 头发送
    pub token: String,
}

