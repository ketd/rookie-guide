use auth::{JwtService, JwtServiceImpl, PasswordService, PasswordServiceImpl};
use common::AppConfig;
use db::{
    TemplateRepository, TemplateRepositoryImpl,
    UserRepository, UserRepositoryImpl,
    UserChecklistRepository, UserChecklistRepositoryImpl,
};
use crate::services::{
    TemplateService, TemplateServiceImpl,
    UserService, UserServiceImpl,
    ChecklistService, ChecklistServiceImpl,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// 应用程序依赖注入容器
/// 
/// ## 设计模式：手动依赖注入
/// 
/// 本模块实现了一个手动的依赖注入模式，使用trait对象和Arc实现线程安全的共享所有权。
/// 所有依赖按照正确的顺序构造，并通过构造函数注入。
/// 
/// ## 架构层次：
/// ```
/// AppModule（应用模块）
///   ├── TemplateService（模板服务）      → 依赖 TemplateRepository
///   ├── UserService（用户服务）          → 依赖 UserRepository, JwtService, PasswordService
///   └── ChecklistService（清单服务）     → 依赖 UserChecklistRepository, TemplateRepository
/// ```
/// 
/// ## 依赖注入的好处：
/// - ✅ **解耦**: 服务不直接依赖具体实现，便于替换和测试
/// - ✅ **可测试**: 可以轻松注入mock对象进行单元测试
/// - ✅ **类型安全**: 编译时检查所有依赖关系
/// - ✅ **线程安全**: 使用Arc实现跨线程共享
pub struct AppModule {
    /// 模板服务：处理阅历模板的创建、查询、搜索等业务逻辑
    pub template_service: Arc<dyn TemplateService>,
    
    /// 用户服务：处理用户注册、登录、认证等业务逻辑
    pub user_service: Arc<dyn UserService>,
    
    /// 清单服务：处理用户清单的fork、进度追踪等业务逻辑
    pub checklist_service: Arc<dyn ChecklistService>,
}

impl AppModule {
    /// 创建一个完整初始化的依赖注入容器
    /// 
    /// 该方法按照依赖层次顺序创建所有服务实例：
    /// 1. **Repository层**（数据访问层）- 负责数据库操作
    /// 2. **Infrastructure层**（基础设施层）- 负责认证、加密等
    /// 3. **Service层**（业务逻辑层）- 负责核心业务逻辑
    /// 
    /// ## 参数
    /// * `db` - SeaORM 数据库连接，用于创建Repository实例
    /// * `config` - 应用配置，包含JWT密钥、过期时间等
    /// 
    /// ## 返回
    /// 返回一个完全初始化的AppModule，所有服务已就绪
    /// 
    /// ## 示例
    /// ```rust
    /// let db = create_sea_orm_connection(&db_url, 5).await?;
    /// let config = AppConfig::from_env()?;
    /// let app_module = AppModule::new(db, config);
    /// 
    /// // 现在可以使用服务了
    /// app_module.user_service.register(dto).await?;
    /// ```
    pub fn new(db: DatabaseConnection, config: AppConfig) -> Self {
        // ==================== 第1层：数据访问层（Repository） ====================
        // Repository负责与数据库交互，执行CRUD操作
        
        // 模板数据访问：负责templates表的所有数据库操作
        let template_repo = Arc::new(TemplateRepositoryImpl::new(db.clone())) 
            as Arc<dyn TemplateRepository>;
        
        // 用户数据访问：负责users表的所有数据库操作
        let user_repo = Arc::new(UserRepositoryImpl::new(db.clone())) 
            as Arc<dyn UserRepository>;
        
        // 清单数据访问：负责user_checklists表的所有数据库操作
        let checklist_repo = Arc::new(UserChecklistRepositoryImpl::new(db.clone())) 
            as Arc<dyn UserChecklistRepository>;

        // ==================== 第2层：基础设施层（Infrastructure） ====================
        // 提供认证、加密等基础功能
        
        // JWT服务：负责生成和验证JWT token
        let jwt_service = Arc::new(JwtServiceImpl::new(
            config.jwt.secret.clone(),
            config.jwt.expiration,
        )) as Arc<dyn JwtService>;
        
        // 密码服务：负责密码的加密和验证（使用bcrypt）
        let password_service = Arc::new(PasswordServiceImpl::new()) 
            as Arc<dyn PasswordService>;

        // ==================== 第3层：业务逻辑层（Service） ====================
        // 实现核心业务逻辑，依赖注入下层服务
        
        // 模板服务：处理模板的创建、搜索、查询等业务逻辑
        let template_service = Arc::new(TemplateServiceImpl::new(
            template_repo.clone()  // 注入：模板数据访问
        )) as Arc<dyn TemplateService>;
        
        // 用户服务：处理用户注册、登录、认证等业务逻辑
        let user_service = Arc::new(UserServiceImpl::new(
            user_repo.clone(),          // 注入：用户数据访问
            jwt_service.clone(),        // 注入：JWT服务
            password_service.clone(),   // 注入：密码服务
        )) as Arc<dyn UserService>;
        
        // 清单服务：处理清单fork、进度追踪等业务逻辑
        let checklist_service = Arc::new(ChecklistServiceImpl::new(
            checklist_repo.clone(),     // 注入：清单数据访问
            template_repo.clone(),      // 注入：模板数据访问（需要读取模板）
        )) as Arc<dyn ChecklistService>;

        // 返回完整的依赖注入容器
        Self {
            template_service,
            user_service,
            checklist_service,
        }
    }
}

