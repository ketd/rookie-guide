/// API处理器模块
/// 
/// 该模块包含所有HTTP请求的处理器（handlers）。
/// 每个子模块对应一组相关的API端点。
/// 
/// ## 模块结构
/// 
/// - `health`: 健康检查端点
/// - `auth`: 用户认证（注册、登录）
/// - `user`: 用户资料管理
/// - `template`: 经验模板CRUD
/// - `checklist`: 用户清单和进度追踪
/// 
/// ## 架构层次
/// 
/// ```
/// Handler层（本模块）
///     ↓ 调用
/// Service层（业务逻辑）
///     ↓ 调用
/// Repository层（数据访问）
///     ↓ 调用
/// 数据库（PostgreSQL）
/// ```
/// 
/// ## 职责
/// 
/// Handlers负责：
/// 1. ✅ 解析HTTP请求（路径参数、查询参数、请求体）
/// 2. ✅ 提取认证信息（JWT token）
/// 3. ✅ 调用业务逻辑层（Service）
/// 4. ✅ 处理错误并转换为HTTP状态码
/// 5. ✅ 返回JSON响应
/// 
/// Handlers不应：
/// 1. ❌ 直接访问数据库
/// 2. ❌ 包含业务逻辑（委托给Service层）
/// 3. ❌ 执行复杂的数据转换（应在Service或Model层）
pub mod health;
pub mod auth;
pub mod user;
pub mod template;
pub mod checklist;

