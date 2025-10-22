/// 数据库访问层（Database Layer）
/// 
/// 该模块负责所有与数据库的交互，包括：
/// - **连接池管理**：创建和配置PostgreSQL连接池
/// - **Repository模式**：为每个实体提供数据访问接口
/// - **SQL执行**：使用SQLx执行类型安全的SQL查询
/// 
/// ## 模块组织
/// 
/// ```
/// db/
/// ├── pool.rs                          # 数据库连接池
/// │   └── create_pool()                # 创建PgPool
/// └── repositories/                    # Repository层
///     ├── template_repository.rs       # 模板数据访问
///     ├── user_repository.rs           # 用户数据访问
///     └── user_checklist_repository.rs # 清单数据访问
/// ```
/// 
/// ## Repository模式
/// 
/// 每个实体都有对应的Repository，提供CRUD操作：
/// 
/// ```rust
/// // Repository trait（接口定义）
/// #[async_trait]
/// pub trait UserRepository: Send + Sync {
///     async fn create(&self, user: User) -> AppResult<User>;
///     async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
///     // ... 其他方法
/// }
/// 
/// // Repository实现（SQLx版本）
/// pub struct UserRepositoryImpl {
///     pool: PgPool,  // 数据库连接池
/// }
/// ```
/// 
/// ## 使用SQLx
/// 
/// - **编译时检查**：SQL语句在编译时验证
/// - **原生异步**：基于tokio的异步I/O
/// - **类型安全**：自动映射到Rust类型
/// - **零成本抽象**：性能接近原生驱动
/// 
/// ## 架构层次
/// 
/// ```
/// Service层（业务逻辑）
///     ↓ 调用
/// Repository层（本模块）
///     ↓ 使用
/// SQLx（SQL执行）
///     ↓ 连接
/// PostgreSQL（数据库）
/// ```

pub mod pool;
pub mod repositories;

// 从pool模块导出创建连接池的函数
pub use pool::create_database_connection;

// 从repositories模块导出所有Repository接口和实现
// - TemplateRepository/TemplateRepositoryImpl: 模板数据访问
// - UserRepository/UserRepositoryImpl: 用户数据访问
// - UserChecklistRepository/UserChecklistRepositoryImpl: 清单数据访问
pub use repositories::{
    TemplateRepository, TemplateRepositoryImpl,
    UserRepository, UserRepositoryImpl,
    UserChecklistRepository, UserChecklistRepositoryImpl,
};

