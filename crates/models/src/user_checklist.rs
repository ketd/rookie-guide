use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
use utoipa::ToSchema;

/// 单个步骤的完成状态
/// 
/// 记录用户清单中每个步骤的完成情况。
/// 
/// ## 字段说明
/// 
/// - `step_index`: 步骤索引（对应模板中的order字段）
/// - `completed`: 是否已完成
/// - `completed_at`: 完成时间（完成时记录，未完成为None）
/// 
/// ## 示例
/// 
/// ```json
/// {
///   "step_index": 0,
///   "completed": true,
///   "completed_at": "2024-10-21T12:34:56Z"
/// }
/// ```
/// 
/// ## 使用场景
/// 
/// - 用户勾选某个步骤时，设置`completed = true`并记录当前时间
/// - 用户取消勾选时，设置`completed = false`并清空时间
/// - 展示完成历史："你在3天前完成了这一步"
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StepProgress {
    /// 步骤索引（从0开始）
    /// 
    /// 对应模板步骤中的`order`字段
    pub step_index: i32,
    
    /// 是否已完成
    /// 
    /// - `true`: 用户已完成这一步
    /// - `false`: 还未完成
    pub completed: bool,
    
    /// 完成时间
    /// 
    /// - `Some(timestamp)`: 完成时记录的时间
    /// - `None`: 还未完成或取消勾选
    /// 
    /// 用于统计："你已经坚持了X天"、"平均每天完成Y步"
    pub completed_at: Option<DateTime<Utc>>,
}

/// 清单整体进度统计
/// 
/// 计算并展示用户清单的完成进度。
/// 
/// ## 字段说明
/// 
/// - `steps`: 所有步骤的完成状态
/// - `total_steps`: 总步骤数
/// - `completed_steps`: 已完成步骤数
/// - `progress_percentage`: 完成百分比（0-100）
/// 
/// ## 示例
/// 
/// ```json
/// {
///   "steps": [...],
///   "total_steps": 10,
///   "completed_steps": 3,
///   "progress_percentage": 30.0
/// }
/// ```
/// 
/// ## 计算逻辑
/// 
/// ```
/// 完成百分比 = (已完成步骤数 / 总步骤数) × 100
/// 
/// 例如：10步中完成了3步
/// → 30.0%
/// ```
/// 
/// ## 前端展示
/// 
/// - 进度条：`width: {progress_percentage}%`
/// - 文字："已完成 3/10 步 (30%)"
/// - 鼓励语："还有7步，加油！"
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChecklistProgress {
    /// 所有步骤的状态详情
    pub steps: Vec<StepProgress>,
    
    /// 总步骤数
    pub total_steps: i32,
    
    /// 已完成的步骤数
    pub completed_steps: i32,
    
    /// 完成百分比（0.0 - 100.0）
    pub progress_percentage: f32,
}

/// 用户清单（数据库实体）
/// 
/// 用户Fork模板后创建的个人清单，用于追踪"第一次"的进度。
/// 
/// ## 核心概念
/// 
/// 1. **清单 vs 模板**:
///    - 模板（Template）：公共的指南，只读
///    - 清单（UserChecklist）：用户的个人副本，可修改进度
/// 
/// 2. **Fork机制**:
///    - 用户看到感兴趣的模板
///    - 点击"开始"按钮Fork模板
///    - 系统创建UserChecklist，复制模板的标题和步骤
///    - 所有步骤初始状态为未完成
/// 
/// 3. **进度追踪**:
///    - 用户逐步勾选完成的步骤
///    - 系统记录完成时间
///    - 实时计算完成百分比
/// 
/// ## 数据库表
/// 
/// 对应表: `user_checklists`
/// 
/// ## 数据流
/// 
/// ```
/// 1. 用户Fork模板
///    → 创建UserChecklist
///    → progress_status初始化为全未完成
/// 
/// 2. 用户勾选步骤
///    → 更新progress_status
///    → 设置completed=true和completed_at
/// 
/// 3. 查看进度
///    → 调用calculate_progress()
///    → 返回统计信息
/// ```
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "user_checklists")]
pub struct Model {
    /// 清单唯一标识
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    
    /// 所属用户ID
    /// 
    /// 一个用户可以有多个清单（多个"第一次"）
    pub user_id: Uuid,
    
    /// 来源模板ID
    /// 
    /// 记录这个清单是从哪个模板Fork的，用于：
    /// - 追溯来源
    /// - 统计模板被使用次数
    /// - 未来可以对比模板更新
    pub source_template_id: Uuid,
    
    /// 清单标题
    /// 
    /// Fork时从模板复制而来，是模板的快照。
    /// 即使模板标题后续修改，清单标题不受影响。
    pub title: String,
    
    /// 进度状态（JSON数组，存储在数据库的JSONB字段）
    /// 
    /// 记录每个步骤的完成情况：
    /// ```json
    /// [
    ///   { "step_index": 0, "completed": true, "completed_at": "..." },
    ///   { "step_index": 1, "completed": false, "completed_at": null }
    /// ]
    /// ```
    #[sea_orm(column_type = "Json")]
    pub progress_status: Json,
    
    /// 清单创建时间（Fork时间）
    pub created_at: DateTime<Utc>,
    
    /// 最后更新时间（最后一次勾选步骤的时间）
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::template::Entity",
        from = "Column::SourceTemplateId",
        to = "super::template::Column::Id"
    )]
    Template,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Template.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// 辅助函数：从 Model 获取步骤进度列表
impl Model {
    pub fn get_progress(&self) -> Result<Vec<StepProgress>, serde_json::Error> {
        serde_json::from_value(self.progress_status.clone())
    }
    
    pub fn set_progress(&mut self, progress: Vec<StepProgress>) -> Result<(), serde_json::Error> {
        self.progress_status = serde_json::to_value(progress)?;
        Ok(())
    }

    /// 计算清单的完成进度
    /// 
    /// ## 返回值
    /// 
    /// `ChecklistProgress`: 包含总步骤数、已完成数、完成百分比等统计信息
    /// 
    /// ## 示例
    /// 
    /// ```rust
    /// let checklist = Model { /* ... */ };
    /// let progress = checklist.calculate_progress()?;
    /// 
    /// println!("进度: {}/{} ({}%)", 
    ///     progress.completed_steps,
    ///     progress.total_steps,
    ///     progress.progress_percentage
    /// );
    /// // 输出: 进度: 3/10 (30%)
    /// ```
    /// 
    /// ## 计算逻辑
    /// 
    /// 1. 统计总步骤数（progress_status.len()）
    /// 2. 统计已完成步骤数（completed=true的数量）
    /// 3. 计算百分比：(已完成 / 总数) × 100
    /// 4. 返回完整的进度信息
    pub fn calculate_progress(&self) -> Result<ChecklistProgress, serde_json::Error> {
        let progress_status = self.get_progress()?;
        
        // 总步骤数
        let total = progress_status.len() as i32;
        
        // 已完成步骤数（筛选completed=true的步骤）
        let completed = progress_status.iter().filter(|s| s.completed).count() as i32;
        
        // 计算完成百分比（避免除以0）
        let percentage = if total > 0 {
            (completed as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        Ok(ChecklistProgress {
            steps: progress_status,
            total_steps: total,
            completed_steps: completed,
            progress_percentage: percentage,
        })
    }
}

/// Fork模板DTO
/// 
/// 用于POST /api/checklists接口，将模板Fork到个人清单。
/// 
/// ## 请求体示例
/// 
/// ```json
/// {
///   "template_id": "550e8400-e29b-41d4-a716-446655440000"
/// }
/// ```
/// 
/// ## 业务逻辑
/// 
/// 1. 验证模板是否存在
/// 2. 获取模板的标题和步骤
/// 3. 创建UserChecklist
/// 4. 初始化所有步骤为未完成状态
/// 5. 返回新创建的清单
#[derive(Debug, Deserialize, ToSchema)]
pub struct ForkTemplateDto {
    /// 要Fork的模板ID
    pub template_id: Uuid,
}

/// 更新步骤状态DTO
/// 
/// 用于PUT /api/checklists/:id/steps接口，更新某个步骤的完成状态。
/// 
/// ## 请求体示例
/// 
/// ```json
/// {
///   "step_index": 0,
///   "completed": true
/// }
/// ```
/// 
/// ## 业务逻辑
/// 
/// 1. 查找指定清单的progress_status
/// 2. 定位step_index对应的步骤
/// 3. 更新completed字段
/// 4. 如果设为完成，记录当前时间到completed_at
/// 5. 如果取消完成，清空completed_at
/// 6. 保存到数据库
/// 7. 返回更新后的清单和新进度
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStepDto {
    /// 要更新的步骤索引（从0开始）
    pub step_index: i32,
    
    /// 新的完成状态
    /// 
    /// - `true`: 标记为已完成
    /// - `false`: 标记为未完成
    pub completed: bool,
}

/// 用户清单响应DTO
/// 
/// API返回给前端的数据结构，包含清单详情和计算好的进度信息。
/// 
/// ## 响应示例
/// 
/// ```json
/// {
///   "checklist": {
///     "id": "uuid",
///     "user_id": "uuid",
///     "source_template_id": "uuid",
///     "title": "第一次在北京租房",
///     "progress_status": [
///       { "step_index": 0, "completed": true, "completed_at": "..." },
///       { "step_index": 1, "completed": false, "completed_at": null }
///     ],
///     "created_at": "...",
///     "updated_at": "..."
///   },
///   "progress": {
///     "steps": [...],
///     "total_steps": 10,
///     "completed_steps": 1,
///     "progress_percentage": 10.0
///   }
/// }
/// ```
/// 
/// ## 设计理由
/// 
/// 为什么不把progress直接放在UserChecklist里？
/// 
/// 1. **关注点分离**: 清单是持久化数据，进度是计算结果
/// 2. **避免冗余**: progress可以从progress_status实时计算
/// 3. **灵活性**: API可以根据需要选择是否返回progress
/// 
/// ## 使用场景
/// 
/// 所有返回清单的API都使用此结构：
/// - GET /api/checklists - 返回用户的所有清单
/// - GET /api/checklists/:id - 返回单个清单详情
/// - POST /api/checklists - Fork模板后返回新清单
/// - PUT /api/checklists/:id/steps - 更新步骤后返回最新清单
#[derive(Debug, Serialize, ToSchema)]
pub struct UserChecklistResponse {
    /// 清单详情
    pub checklist: Model,
    
    /// 进度统计（实时计算）
    pub progress: ChecklistProgress,
}

