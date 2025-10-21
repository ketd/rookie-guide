/// Repository层模块
/// 
/// 实现Repository模式，为每个实体提供数据访问接口。
/// 
/// ## Repository模式的优势
/// 
/// 1. **抽象数据访问**：Service层不需要知道数据来源（SQL、NoSQL、缓存等）
/// 2. **易于测试**：可以轻松mock Repository进行单元测试
/// 3. **业务逻辑分离**：将SQL操作从业务逻辑中分离
/// 4. **可替换实现**：未来可以切换数据库或添加缓存层
/// 
/// ## 模块结构
/// 
/// ```
/// repositories/
/// ├── user_repository.rs           # 用户数据访问
/// │   ├── UserRepository trait     # 接口定义
/// │   └── UserRepositoryImpl       # SQLx实现
/// ├── template_repository.rs       # 模板数据访问
/// │   ├── TemplateRepository trait
/// │   └── TemplateRepositoryImpl
/// └── user_checklist_repository.rs # 清单数据访问
///     ├── UserChecklistRepository trait
///     └── UserChecklistRepositoryImpl
/// ```
/// 
/// ## 使用示例
/// 
/// ```rust
/// // 在Service层中使用Repository
/// pub struct UserServiceImpl {
///     user_repo: Arc<dyn UserRepository>,  // 依赖注入
/// }
/// 
/// impl UserServiceImpl {
///     async fn get_user(&self, id: Uuid) -> AppResult<UserProfile> {
///         let user = self.user_repo
///             .find_by_id(id)
///             .await?
///             .ok_or_else(|| AppError::NotFound("用户不存在".into()))?;
///         
///         Ok(user.into())
///     }
/// }
/// ```

mod template_repository;
mod user_repository;
mod user_checklist_repository;

// 导出所有Repository接口和实现
pub use template_repository::{TemplateRepository, TemplateRepositoryImpl};
pub use user_repository::{UserRepository, UserRepositoryImpl};
pub use user_checklist_repository::{UserChecklistRepository, UserChecklistRepositoryImpl};

