/// 数据模型（Models）模块
/// 
/// 该模块定义了应用程序的所有数据结构，包括：
/// - **数据库实体**：对应数据库表的结构体（User、Template、UserChecklist）
/// - **DTO（Data Transfer Objects）**：API请求/响应的数据结构
/// - **领域模型**：业务逻辑相关的数据类型
/// 
/// ## 模块组织
/// 
/// ```
/// models/
/// ├── user.rs              # 用户相关模型
/// │   ├── User             # 用户实体
/// │   ├── UserProfile      # 用户公开资料
/// │   └── RegisterDto、LoginDto等
/// ├── template.rs          # 模板相关模型
/// │   ├── Template         # 模板实体
/// │   ├── TemplateStep     # 模板步骤
/// │   └── CreateTemplateDto等
/// └── user_checklist.rs    # 清单相关模型
///     ├── UserChecklist    # 用户清单实体
///     ├── StepProgress     # 步骤进度
///     └── ForkTemplateDto等
/// ```
/// 
/// ## 设计原则
/// 
/// 1. **关注点分离**：
///    - 数据库实体（User）vs API响应（UserProfile）
///    - 持久化数据 vs 传输数据
/// 
/// 2. **类型安全**：
///    - 使用UUID代替字符串ID
///    - 使用枚举（LocationTag）代替魔术字符串
///    - 使用validator进行输入验证
/// 
/// 3. **自文档化**：
///    - 结构体和字段都有详细注释
///    - 包含使用示例和JSON示例
/// 
/// ## 使用示例
/// 
/// ```rust
/// use models::{User, Template, RegisterDto};
/// 
/// // 创建注册DTO
/// let dto = RegisterDto {
///     phone: Some("13800138000".to_string()),
///     email: None,
///     password: "password123".to_string(),
///     nickname: "张三".to_string(),
/// };
/// 
/// // 使用模板
/// let template = Template { /* ... */ };
/// println!("模板标题: {}", template.title);
/// ```

pub mod template;
pub mod user;
pub mod user_checklist;

// ==================== SeaORM 实体导出 ====================
// SeaORM 生成的实体类型
pub use user::Entity as UserEntity;
pub use template::Entity as TemplateEntity;
pub use user_checklist::Entity as UserChecklistEntity;

// 用于查询构建的列定义
pub use user::Column as UserColumn;
pub use template::Column as TemplateColumn;
pub use user_checklist::Column as UserChecklistColumn;

// ==================== 模板相关导出 ====================
// - Model: 经验模板实体（SeaORM Model）
// - TemplateStep: 模板中的单个步骤
// - LocationTag: 地理位置标签枚举
// - CreateTemplateDto: 创建模板DTO
// - UpdateTemplateDto: 更新模板DTO
// - TemplateSearchQuery: 模板搜索查询DTO
pub use template::{
    Model as Template,
    TemplateStep, LocationTag, 
    CreateTemplateDto, UpdateTemplateDto, TemplateSearchQuery
};

// ==================== 用户相关导出 ====================
// - Model: 用户数据库实体（SeaORM Model）
// - UserProfile: 用户公开资料（不含敏感信息）
// - RegisterDto: 用户注册DTO
// - LoginDto: 用户登录DTO
// - UpdateProfileDto: 更新用户资料DTO
// - AuthResponse: 认证响应（包含用户信息和JWT token）
pub use user::{
    Model as User,
    UserProfile, 
    RegisterDto, LoginDto, UpdateProfileDto, AuthResponse
};

// ==================== 用户清单相关导出 ====================
// - Model: 用户清单实体（SeaORM Model）
// - ChecklistProgress: 清单整体进度统计
// - StepProgress: 单个步骤进度
// - ForkTemplateDto: Fork模板DTO
// - UpdateStepDto: 更新步骤DTO
// - UserChecklistResponse: 用户清单响应（包含清单和进度）
pub use user_checklist::{
    Model as UserChecklist,
    ChecklistProgress, StepProgress,
    ForkTemplateDto, UpdateStepDto, UserChecklistResponse
};

