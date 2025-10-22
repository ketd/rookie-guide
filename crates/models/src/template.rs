use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

/// 地理位置标签（枚举类型，SeaORM 存储为字符串）
/// 
/// 用于标识模板的地域属性，支持通用模板和城市特定模板。
/// 
/// ## 设计理念
/// 
/// 同一个"第一次"体验在不同城市可能有显著差异：
/// - "第一次租房"：北京和上海的流程、注意事项不同
/// - "第一次办证"：各地政策和办事地点不同
/// 
/// ## 标签层级
/// 
/// - `CN`: 全国通用模板（如"第一次面试"）
/// - `CN-{城市}`: 城市特定模板（如"第一次在北京租房"）
/// 
/// ## 使用示例
/// 
/// ```rust
/// // 创建通用模板
/// let tag = LocationTag::China;  // "CN"
/// 
/// // 创建北京专属模板
/// let tag = LocationTag::Beijing;  // "CN-BJ"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum LocationTag {
    /// 全国通用模板
    /// 
    /// 适用于不受地域限制的经验，如：
    /// - 第一次求职面试
    /// - 第一次理财
    /// - 第一次独立生活
    #[serde(rename = "CN")]
    China,
    
    /// 北京专属模板（CN-BJ）
    /// 
    /// 例如：第一次在北京租房、办理北京居住证
    #[serde(rename = "CN-BJ")]
    Beijing,
    
    /// 上海专属模板（CN-SH）
    #[serde(rename = "CN-SH")]
    Shanghai,
    
    /// 广州专属模板（CN-GZ）
    #[serde(rename = "CN-GZ")]
    Guangzhou,
    
    /// 深圳专属模板（CN-SZ）
    #[serde(rename = "CN-SZ")]
    Shenzhen,
    
    // 未来可以继续添加更多城市
}

impl std::fmt::Display for LocationTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocationTag::China => write!(f, "CN"),
            LocationTag::Beijing => write!(f, "CN-BJ"),
            LocationTag::Shanghai => write!(f, "CN-SH"),
            LocationTag::Guangzhou => write!(f, "CN-GZ"),
            LocationTag::Shenzhen => write!(f, "CN-SZ"),
        }
    }
}

/// 模板步骤（单个步骤的定义）
/// 
/// 每个经验模板由多个步骤组成，用户fork后会逐步完成这些步骤。
/// 
/// ## 字段说明
/// 
/// - `title`: 步骤标题（1-500字符）
/// - `description`: 步骤详细说明（可选）
/// - `order`: 步骤顺序（从0开始）
/// 
/// ## 示例
/// 
/// ```json
/// {
///   "title": "确定租房预算和区域",
///   "description": "根据工作地点和收入，确定可接受的租金范围和通勤距离",
///   "order": 0
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct TemplateStep {
    /// 步骤标题（简短描述要做什么）
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    
    /// 步骤详细说明（如何做、注意事项等）
    pub description: Option<String>,
    
    /// 步骤在清单中的顺序（从0开始）
    /// 
    /// 建议按照实际操作的时间顺序排列
    pub order: i32,
}

/// 经验模板（数据库实体）
/// 
/// 模板是由官方团队或社区用户创建的"第一次"经验指南，
/// 包含完成某项任务所需的所有步骤。
/// 
/// ## 核心概念
/// 
/// 1. **模板 vs 清单**:
///    - 模板：公共的、不可变的指南（Template）
///    - 清单：用户Fork模板后的个人副本（UserChecklist）
/// 
/// 2. **官方 vs 用户创建**:
///    - `is_official = true`: 官方团队创建，质量保证
///    - `is_official = false`: 用户创建，社区贡献
/// 
/// ## 数据库表
/// 
/// 对应表: `templates`
/// 
/// ## 使用流程
/// 
/// ```
/// 1. 官方/用户创建模板（Template）
///    ↓
/// 2. 用户浏览并选择模板
///    ↓
/// 3. 用户Fork模板到个人清单（UserChecklist）
///    ↓
/// 4. 用户逐步完成清单中的步骤
/// ```
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "templates")]
pub struct Model {
    /// 模板唯一标识
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    
    /// 模板标题
    /// 
    /// 建议格式：
    /// - 通用模板："第一次{动作}"（如"第一次面试"）
    /// - 地域模板："第一次在{城市}{动作}"（如"第一次在北京租房"）
    pub title: String,
    
    /// 模板描述（介绍这个模板的用途和适用场景）
    pub description: String,
    
    /// 地理位置标签（存储为文本，可解析为LocationTag）
    /// 
    /// 值：
    /// - "CN": 全国通用
    /// - "CN-BJ": 北京专属
    /// - "CN-SH": 上海专属
    pub location_tag: String,
    
    /// 步骤列表（JSON数组，存储在数据库的JSONB字段）
    /// 
    /// 包含完成该经验所需的所有步骤，按order字段排序
    #[sea_orm(column_type = "Json")]
    pub steps: Json,
    
    /// 父模板ID（用于模板继承）
    /// 
    /// **未来功能**：
    /// - 通用模板（CN）作为父模板
    /// - 城市模板（CN-BJ）继承父模板并添加本地化步骤
    /// - V0.0.1版本暂不实现继承逻辑
    pub parent_id: Option<Uuid>,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    
    /// 创建者用户ID
    /// 
    /// 用于：
    /// - 标识模板来源
    /// - 统计用户贡献
    /// - 权限控制（创建者可编辑）
    pub created_by: Uuid,
    
    /// 是否为官方模板
    /// 
    /// - `true`: 官方团队创建，经过审核，质量保证
    /// - `false`: 用户创建，社区贡献
    /// 
    /// 官方模板会优先展示，并有特殊标识
    pub is_official: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(has_many = "super::user_checklist::Entity")]
    Checklists,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::user_checklist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Checklists.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// 辅助函数：从 Model 获取步骤列表
impl Model {
    pub fn get_steps(&self) -> Result<Vec<TemplateStep>, serde_json::Error> {
        serde_json::from_value(self.steps.clone())
    }
    
    pub fn set_steps(&mut self, steps: Vec<TemplateStep>) -> Result<(), serde_json::Error> {
        self.steps = serde_json::to_value(steps)?;
        Ok(())
    }
}

/// 创建模板DTO
/// 
/// 用于POST /api/templates接口的请求体
/// 
/// ## 验证规则
/// 
/// - `title`: 1-200字符
/// - `description`: 1-2000字符
/// - `location_tag`: 有效的地理标签（CN、CN-BJ等）
/// - `steps`: 至少包含1个步骤
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTemplateDto {
    /// 模板标题
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    
    /// 模板描述
    #[validate(length(min = 1, max = 2000))]
    pub description: String,
    
    /// 地理位置标签
    pub location_tag: String,
    
    /// 步骤列表（至少1个）
    #[validate(length(min = 1))]
    pub steps: Vec<TemplateStep>,
    
    /// 父模板ID（可选，用于模板继承）
    pub parent_id: Option<Uuid>,
}

/// 更新模板DTO
/// 
/// 用于PUT /api/templates/:id接口的请求体
/// 
/// 所有字段都是可选的，只更新提供的字段。
/// 
/// ## 注意事项
/// 
/// - V0.0.1版本暂未实现此功能
/// - 更新模板会影响所有基于该模板的清单吗？
///   答：不会，Fork的是快照，不受模板更新影响
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateTemplateDto {
    /// 新标题
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    
    /// 新描述
    #[validate(length(min = 1, max = 2000))]
    pub description: Option<String>,
    
    /// 新地理标签
    pub location_tag: Option<String>,
    
    /// 新步骤列表
    pub steps: Option<Vec<TemplateStep>>,
}

/// 模板搜索查询DTO
/// 
/// 用于GET /api/templates/search接口的查询参数
/// 
/// ## 查询参数
/// 
/// - `keyword`: 关键词（在标题和描述中搜索）
/// - `location_tag`: 地理标签过滤
/// - `page`: 页码（默认1）
/// - `page_size`: 每页数量（默认20）
/// 
/// ## 示例
/// 
/// ```
/// # 搜索北京的租房模板
/// GET /api/templates/search?keyword=租房&location_tag=CN-BJ
/// 
/// # 搜索所有面试相关模板
/// GET /api/templates/search?keyword=面试
/// 
/// # 分页获取第2页
/// GET /api/templates/search?page=2&page_size=10
/// ```
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct TemplateSearchQuery {
    /// 搜索关键词（模糊匹配标题和描述）
    pub keyword: Option<String>,
    
    /// 地理标签过滤（精确匹配）
    /// 
    /// 搜索某城市时，会同时返回该城市和通用（CN）的模板
    pub location_tag: Option<String>,
    
    /// 页码（从1开始）
    pub page: Option<i32>,
    
    /// 每页数量
    pub page_size: Option<i32>,
}

