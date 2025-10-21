use async_trait::async_trait;
use common::AppResult;
use models::{Template, CreateTemplateDto, TemplateSearchQuery, TemplateEntity, TemplateColumn};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set, ColumnTrait, ActiveModelTrait};
use uuid::Uuid;

/// 模板Repository接口
/// 
/// 定义了所有模板相关的数据访问操作。
/// 
/// ## 职责
/// 
/// - 创建新模板
/// - 查询模板（按ID、地理位置、关键词搜索）
/// - 分页列出模板
/// 
/// ## 使用场景
/// 
/// 在TemplateService中通过依赖注入使用：
/// ```rust
/// pub struct TemplateServiceImpl {
///     template_repo: Arc<dyn TemplateRepository>,
/// }
/// ```
#[async_trait]
pub trait TemplateRepository: Send + Sync {
    /// 创建新模板
    /// 
    /// ## 参数
    /// - `dto`: 创建模板的数据传输对象
    /// - `created_by`: 创建者用户ID
    /// 
    /// ## 返回值
    /// 创建成功的模板实体（包含生成的UUID和时间戳）
    async fn create(&self, dto: CreateTemplateDto, created_by: Uuid) -> AppResult<Template>;
    
    /// 根据ID查找模板
    /// 
    /// ## 参数
    /// - `id`: 模板UUID
    /// 
    /// ## 返回值
    /// - `Some(Template)`: 找到模板
    /// - `None`: 模板不存在
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Template>>;
    
    /// 搜索模板
    /// 
    /// 支持关键词搜索、地理位置过滤和分页。
    /// 
    /// ## 参数
    /// - `query`: 搜索查询对象（包含keyword、location_tag、page、page_size）
    /// 
    /// ## 返回值
    /// 匹配的模板列表，按创建时间倒序排列
    async fn search(&self, query: TemplateSearchQuery) -> AppResult<Vec<Template>>;
    
    /// 根据地理位置查找模板
    /// 
    /// 查找指定地理标签的模板，同时包含通用模板（CN）。
    /// 
    /// ## 参数
    /// - `location_tag`: 地理位置标签（如"CN-BJ"）
    /// 
    /// ## 返回值
    /// 该地理位置的模板 + 通用模板，按创建时间倒序
    async fn find_by_location(&self, location_tag: String) -> AppResult<Vec<Template>>;
    
    /// 分页列出所有模板
    /// 
    /// ## 参数
    /// - `page`: 页码（从1开始）
    /// - `page_size`: 每页数量
    /// 
    /// ## 返回值
    /// 指定页的模板列表，按创建时间倒序排列
    async fn list_all(&self, page: i32, page_size: i32) -> AppResult<Vec<Template>>;
}

/// 模板Repository的SeaORM实现
/// 
/// 使用PostgreSQL存储模板数据，步骤以JSONB格式存储。
#[derive(Clone)]
pub struct TemplateRepositoryImpl {
    /// SeaORM 数据库连接
    db: DatabaseConnection,
}

impl TemplateRepositoryImpl {
    /// 创建新的TemplateRepository实例
    /// 
    /// ## 参数
    /// - `db`: SeaORM 数据库连接
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TemplateRepository for TemplateRepositoryImpl {
    /// 创建新模板
    /// 
    /// ## SeaORM ActiveModel 模式
    /// 
    /// 使用 ActiveModel 创建新记录：
    /// 1. 创建 ActiveModel 实例
    /// 2. 使用 Set() 包装所有字段值
    /// 3. 调用 insert() 插入数据库
    /// 4. 返回插入的实体
    /// 
    /// ### 注意事项
    /// - `steps` 字段存储为 JSONB，需要先序列化为 JSON
    /// - `is_official` 默认为 false（用户创建的模板）
    /// - `id` 使用 UUID v4 自动生成
    /// - `created_at` 和 `updated_at` 都设置为当前时间
    async fn create(&self, dto: CreateTemplateDto, created_by: Uuid) -> AppResult<Template> {
        use models::template::ActiveModel;
        
        // 生成新的 UUID
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        // 将步骤列表序列化为 JSON（存储到 JSONB 字段）
        let steps_json = serde_json::to_value(&dto.steps)?;
        
        // 创建 ActiveModel（SeaORM 的插入/更新模型）
        let active_model = ActiveModel {
            id: Set(id),
            title: Set(dto.title),
            description: Set(dto.description),
            location_tag: Set(dto.location_tag),
            steps: Set(steps_json),
            parent_id: Set(dto.parent_id),
            created_at: Set(now),
            updated_at: Set(now),
            created_by: Set(created_by),
            is_official: Set(false), // 默认非官方模板（用户创建）
        };

        // 插入数据库并返回创建的模板
        let template = active_model.insert(&self.db).await?;
        Ok(template)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Template>> {
        let template = TemplateEntity::find_by_id(id)
            .one(&self.db)
            .await?;

        Ok(template)
    }

    /// 搜索模板
    /// 
    /// ## SeaORM 查询逻辑
    /// 
    /// 动态构建查询条件，支持：
    /// 1. **关键词搜索**：使用LIKE模糊匹配标题和描述
    /// 2. **地理位置过滤**：查找指定地理位置 + 通用模板（CN）
    /// 3. **分页**：使用offset和limit
    /// 
    /// ### SQL示例（有关键词和位置）
    /// ```sql
    /// SELECT * FROM templates
    /// WHERE (title LIKE '%租房%' OR description LIKE '%租房%')
    ///   AND (location_tag = 'CN-BJ' OR location_tag = 'CN')
    /// ORDER BY created_at DESC
    /// LIMIT 20 OFFSET 0;
    /// ```
    /// 
    /// ### SQL示例（仅分页）
    /// ```sql
    /// SELECT * FROM templates
    /// ORDER BY created_at DESC
    /// LIMIT 20 OFFSET 20;  -- 第2页
    /// ```
    async fn search(&self, query: TemplateSearchQuery) -> AppResult<Vec<Template>> {
        // 分页参数（默认第1页，每页20条）
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = ((page - 1) * page_size) as u64;
        
        // 开始构建查询
        let mut query_builder = TemplateEntity::find();
        
        // 关键词搜索（模糊匹配标题和描述）
        // 使用 OR 条件：title LIKE '%keyword%' OR description LIKE '%keyword%'
        if let Some(keyword) = query.keyword {
            let pattern = format!("%{}%", keyword);
            query_builder = query_builder.filter(
                sea_orm::Condition::any()
                    .add(TemplateColumn::Title.like(&pattern))
                    .add(TemplateColumn::Description.like(&pattern))
            );
        }
        
        // 地理位置过滤
        // 查找该地理位置 OR 通用模板（CN）
        // 例如：查询北京模板时，返回 CN-BJ 和 CN 的模板
        if let Some(location_tag) = query.location_tag {
            query_builder = query_builder.filter(
                sea_orm::Condition::any()
                    .add(TemplateColumn::LocationTag.eq(&location_tag))
                    .add(TemplateColumn::LocationTag.eq("CN"))
            );
        }
        
        // 按创建时间倒序排列，应用分页
        let templates = query_builder
            .order_by_desc(TemplateColumn::CreatedAt)
            .offset(offset)
            .limit(page_size as u64)
            .all(&self.db)
            .await?;

        Ok(templates)
    }

    /// 根据地理位置查找模板
    /// 
    /// ## 地理位置查询逻辑
    /// 
    /// 查询指定地理位置的模板时，同时返回通用模板（CN）。
    /// 
    /// ### 设计理由
    /// 
    /// 用户在北京查看"第一次租房"模板时，应该看到：
    /// - CN-BJ（北京专属）的租房模板
    /// - CN（通用）的租房模板
    /// 
    /// ### SQL示例
    /// ```sql
    /// SELECT * FROM templates
    /// WHERE location_tag = 'CN-BJ' OR location_tag = 'CN'
    /// ORDER BY created_at DESC;
    /// ```
    async fn find_by_location(&self, location_tag: String) -> AppResult<Vec<Template>> {
        let templates = TemplateEntity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(TemplateColumn::LocationTag.eq(&location_tag))
                    .add(TemplateColumn::LocationTag.eq("CN"))
            )
            .order_by_desc(TemplateColumn::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(templates)
    }

    async fn list_all(&self, page: i32, page_size: i32) -> AppResult<Vec<Template>> {
        let offset = ((page - 1) * page_size) as u64;
        
        let templates = TemplateEntity::find()
            .order_by_desc(TemplateColumn::CreatedAt)
            .offset(offset)
            .limit(page_size as u64)
            .all(&self.db)
            .await?;

        Ok(templates)
    }
}
