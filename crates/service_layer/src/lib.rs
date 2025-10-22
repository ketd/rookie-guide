/// 业务逻辑层（Service Layer）
/// 
/// 该模块包含所有业务逻辑的实现，位于API层和数据访问层之间。
/// 
/// ## 架构层次
/// 
/// ```
/// API Handler层
///     ↓ 调用
/// Service层（本模块）← 业务逻辑
///     ↓ 调用
/// Repository层 ← 数据访问
///     ↓ 调用
/// 数据库
/// ```
/// 
/// ## 职责
/// 
/// Service层负责：
/// 1. ✅ 实现业务逻辑和规则
/// 2. ✅ 协调多个Repository的操作
/// 3. ✅ 处理事务和数据一致性
/// 4. ✅ 转换数据模型（Entity → DTO）
/// 5. ✅ 验证业务规则
/// 
/// Service层不应：
/// 1. ❌ 直接处理HTTP请求/响应
/// 2. ❌ 包含SQL语句
/// 3. ❌ 访问外部API（应委托给专门的client）
/// 
/// ## 模块结构
/// 
/// - `services/`: 各业务模块的Service实现
///   - `user_service`: 用户注册、登录、资料管理
///   - `template_service`: 模板CRUD和搜索
///   - `checklist_service`: 清单Fork和进度追踪
/// - `di`: 依赖注入容器（AppModule）
/// 
/// ## 依赖注入
/// 
/// 使用手动DI模式：
/// ```rust
/// let app_module = AppModule::new(pool, config);
/// let template_service = &app_module.template_service;
/// ```

pub mod services;
pub mod di;

pub use services::{
    TemplateService,
    UserService,
    ChecklistService,
};
pub use di::AppModule;

