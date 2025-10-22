use async_trait::async_trait;
use common::AppResult;
use models::{UserChecklist, StepProgress, Template, UserChecklistEntity, UserChecklistColumn};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set, ColumnTrait, IntoActiveModel, ActiveModelTrait};
use uuid::Uuid;

/// 用户清单Repository接口
/// 
/// 定义了用户清单相关的数据访问操作。
/// 
/// ## 核心功能
/// 
/// - Fork模板创建清单
/// - 查询用户的所有清单
/// - 更新步骤完成状态
/// 
/// ## Fork机制
/// 
/// 用户Fork模板时，会创建清单的"快照"：
/// 1. 复制模板的标题和步骤
/// 2. 初始化所有步骤为未完成状态
/// 3. 记录来源模板ID（source_template_id）
/// 4. 后续模板修改不影响已创建的清单
#[async_trait]
pub trait UserChecklistRepository: Send + Sync {
    /// 从模板创建用户清单（Fork操作）
    /// 
    /// ## 参数
    /// - `user_id`: 用户ID
    /// - `template`: 要Fork的模板
    /// 
    /// ## 返回值
    /// 新创建的用户清单（所有步骤初始化为未完成）
    async fn create_from_template(&self, user_id: Uuid, template: &Template) -> AppResult<UserChecklist>;
    
    /// 根据ID查找清单
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<UserChecklist>>;
    
    /// 查找用户的所有清单
    /// 
    /// 按创建时间倒序排列
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<UserChecklist>>;
    
    /// 更新步骤的完成状态
    /// 
    /// ## 参数
    /// - `checklist_id`: 清单ID
    /// - `step_index`: 步骤索引（从0开始）
    /// - `completed`: 新的完成状态
    /// 
    /// ## 逻辑
    /// 1. 读取当前清单
    /// 2. 在内存中更新指定步骤的状态
    /// 3. 如果设为完成，记录当前时间
    /// 4. 保存整个progress_status到数据库
    async fn update_step_status(&self, checklist_id: Uuid, step_index: i32, completed: bool) -> AppResult<UserChecklist>;
}

/// 用户清单Repository的SeaORM实现
#[derive(Clone)]
pub struct UserChecklistRepositoryImpl {
    db: DatabaseConnection,
}

impl UserChecklistRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserChecklistRepository for UserChecklistRepositoryImpl {
    /// Fork模板到用户清单
    /// 
    /// ## Fork机制（快照模式）
    /// 
    /// Fork时创建模板的"快照"，步骤存储在清单中：
    /// - ✅ 标题复制：清单独立存储标题
    /// - ✅ 进度独立：每个清单有自己的进度状态
    /// - ✅ 不受影响：模板后续修改不影响已创建的清单
    /// 
    /// ### 初始化进度状态
    /// 
    /// 为模板的每个步骤创建 StepProgress：
    /// ```json
    /// [
    ///   { "step_index": 0, "completed": false, "completed_at": null },
    ///   { "step_index": 1, "completed": false, "completed_at": null },
    ///   ...
    /// ]
    /// ```
    /// 
    /// ### SQL示例
    /// ```sql
    /// INSERT INTO user_checklists (
    ///   id, user_id, source_template_id, title, 
    ///   progress_status, created_at, updated_at
    /// ) VALUES (
    ///   $1, $2, $3, $4, 
    ///   '[{"step_index": 0, "completed": false, ...}]'::jsonb,
    ///   $5, $6
    /// ) RETURNING *;
    /// ```
    async fn create_from_template(&self, user_id: Uuid, template: &Template) -> AppResult<UserChecklist> {
        use models::user_checklist::ActiveModel;
        
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        // 获取模板步骤并初始化进度
        let template_steps = template.get_steps()?;
        let progress_status: Vec<StepProgress> = template_steps
            .iter()
            .enumerate()
            .map(|(index, _step)| StepProgress {
                step_index: index as i32,
                completed: false,
                completed_at: None,
            })
            .collect();
        
        // 序列化进度状态为 JSON
        let progress_json = serde_json::to_value(&progress_status)?;
        
        let active_model = ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
            source_template_id: Set(template.id),
            title: Set(template.title.clone()),
            progress_status: Set(progress_json),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let checklist = active_model.insert(&self.db).await?;
        Ok(checklist)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<UserChecklist>> {
        let checklist = UserChecklistEntity::find_by_id(id)
            .one(&self.db)
            .await?;

        Ok(checklist)
    }

    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<UserChecklist>> {
        let checklists = UserChecklistEntity::find()
            .filter(UserChecklistColumn::UserId.eq(user_id))
            .order_by_desc(UserChecklistColumn::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(checklists)
    }

    /// 更新步骤状态
    /// 
    /// ## JSONB字段更新流程
    /// 
    /// PostgreSQL的JSONB字段不能直接部分更新，需要：
    /// 1. **读取**：从数据库读取整个 progress_status JSONB
    /// 2. **反序列化**：将 JSON 转换为 Vec<StepProgress>
    /// 3. **修改**：在内存中更新指定步骤
    /// 4. **序列化**：将 Vec 转换回 JSON
    /// 5. **保存**：更新整个 progress_status 字段
    /// 
    /// ### 更新逻辑
    /// 
    /// - 如果 `completed = true`：设置 `completed_at` 为当前时间
    /// - 如果 `completed = false`：清空 `completed_at`
    /// - 同时更新清单的 `updated_at` 字段
    /// 
    /// ### SQL示例
    /// ```sql
    /// UPDATE user_checklists
    /// SET progress_status = '[
    ///   {"step_index": 0, "completed": true, "completed_at": "2024-10-21T10:00:00Z"},
    ///   {"step_index": 1, "completed": false, "completed_at": null}
    /// ]'::jsonb,
    /// updated_at = NOW()
    /// WHERE id = $1
    /// RETURNING *;
    /// ```
    async fn update_step_status(&self, checklist_id: Uuid, step_index: i32, completed: bool) -> AppResult<UserChecklist> {
        // 查找清单
        let checklist = UserChecklistEntity::find_by_id(checklist_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| common::AppError::NotFound("Checklist not found".to_string()))?;
        
        // 获取当前进度状态
        let mut progress_status = checklist.get_progress()?;
        
        // 查找并更新指定步骤
        let step = progress_status
            .iter_mut()
            .find(|s| s.step_index == step_index)
            .ok_or_else(|| common::AppError::NotFound(format!("Step {} not found", step_index)))?;
        
        step.completed = completed;
        step.completed_at = if completed {
            Some(chrono::Utc::now())
        } else {
            None
        };
        
        // 序列化更新后的进度
        let progress_json = serde_json::to_value(&progress_status)?;
        
        // 更新数据库
        let mut active_model = checklist.into_active_model();
        active_model.progress_status = Set(progress_json);
        active_model.updated_at = Set(chrono::Utc::now());
        
        let updated_checklist = active_model.update(&self.db).await?;
        
        Ok(updated_checklist)
    }
}
