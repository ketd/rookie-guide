use async_trait::async_trait;
use common::AppResult;
use models::{User, RegisterDto, UpdateProfileDto, UserEntity, UserColumn};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set, ColumnTrait, ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;

/// 用户Repository接口
/// 
/// 定义了所有用户相关的数据访问操作。
/// 
/// ## 职责
/// 
/// - 创建新用户（注册）
/// - 查询用户（按ID、手机号、邮箱）
/// - 更新用户资料
/// 
/// ## 安全性
/// 
/// - 密码必须已经过bcrypt加密才能传入create方法
/// - 所有查询方法都返回完整的User对象（包含password_hash）
/// - 业务层需要使用UserProfile过滤敏感信息
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// 创建新用户
    /// 
    /// ## 参数
    /// - `dto`: 注册数据（手机号/邮箱、明文密码、昵称）
    /// - `password_hash`: 已加密的密码哈希（bcrypt）
    /// 
    /// ## 返回值
    /// 创建成功的用户实体
    /// 
    /// ## 注意
    /// 调用前必须先使用bcrypt加密密码！
    async fn create(&self, dto: RegisterDto, password_hash: String) -> AppResult<User>;
    
    /// 根据ID查找用户
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
    
    /// 根据手机号查找用户
    /// 
    /// 用于登录验证
    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>>;
    
    /// 根据邮箱查找用户
    /// 
    /// 用于登录验证
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    
    /// 更新用户资料
    /// 
    /// 动态更新：只更新DTO中提供的字段。
    async fn update_profile(&self, user_id: Uuid, dto: UpdateProfileDto) -> AppResult<User>;
}

/// 用户Repository的SeaORM实现
#[derive(Clone)]
pub struct UserRepositoryImpl {
    db: DatabaseConnection,
}

impl UserRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn create(&self, dto: RegisterDto, password_hash: String) -> AppResult<User> {
        use models::user::ActiveModel;
        
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        let active_model = ActiveModel {
            id: Set(id),
            phone: Set(dto.phone),
            email: Set(dto.email),
            password_hash: Set(password_hash),
            nickname: Set(dto.nickname),
            avatar_url: Set(None),
            home_city: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let user = active_model.insert(&self.db).await?;
        Ok(user)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let user = UserEntity::find_by_id(id)
            .one(&self.db)
            .await?;

        Ok(user)
    }

    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>> {
        let user = UserEntity::find()
            .filter(UserColumn::Phone.eq(phone))
            .one(&self.db)
            .await?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = UserEntity::find()
            .filter(UserColumn::Email.eq(email))
            .one(&self.db)
            .await?;

        Ok(user)
    }

    /// 更新用户资料
    /// 
    /// ## SeaORM 实现
    /// 
    /// 使用 SeaORM 的 ActiveModel 进行动态更新。
    /// 只更新 DTO 中提供的字段，未提供的字段保持不变。
    async fn update_profile(&self, user_id: Uuid, dto: UpdateProfileDto) -> AppResult<User> {
        use models::user::ActiveModel;
        
        // 先查找用户
        let user = UserEntity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| common::AppError::NotFound("User not found".to_string()))?;
        
        // 将模型转换为 ActiveModel
        let mut active_model: ActiveModel = user.into_active_model();
        
        // 动态更新字段
        if let Some(nickname) = dto.nickname {
            active_model.nickname = Set(nickname);
        }
        
        if let Some(avatar_url) = dto.avatar_url {
            active_model.avatar_url = Set(Some(avatar_url));
        }
        
        if let Some(home_city) = dto.home_city {
            active_model.home_city = Set(Some(home_city));
        }
        
        // 更新 updated_at
        active_model.updated_at = Set(chrono::Utc::now());
        
        // 保存更新
        let updated_user = active_model.update(&self.db).await?;
        
        Ok(updated_user)
    }
}
