use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use models::{Template, CreateTemplateDto, TemplateSearchQuery};
use common::ApiResponse;
use crate::{middleware::CurrentUser, state::AppState};
use uuid::Uuid;

/// 列出所有模板（分页）
/// 
/// ## 端点
/// GET /api/templates?page=1&page_size=20
/// 
/// ## 查询参数
/// - `page`: 页码（可选，默认1）
/// - `page_size`: 每页数量（可选，默认20）
/// 
/// ## 认证
/// 无需认证（公开接口）
/// 
/// ## 响应
/// - 200 OK: 返回模板列表
/// - 500 Internal Server Error: 服务器错误
/// 
/// ## 响应示例
/// ```json
/// [
///   {
///     "id": "uuid",
///     "title": "第一次在北京租房整租指南",
///     "description": "详细的北京租房步骤清单",
///     "location_tag": "CN-BJ",
///     "steps": [...],
///     "created_by": "uuid",
///     "is_official": true
///   }
/// ]
/// ```
/// 
/// ## 业务逻辑
/// 1. 提取分页参数（默认第1页，每页20条）
/// 2. 从数据库查询模板列表
/// 3. 按创建时间倒序排列
/// 4. 返回指定页的模板
/// 
/// ## 使用场景
/// - 首页展示所有可用模板
/// - 浏览模板库
/// - 分页加载模板列表
#[utoipa::path(
    get,
    path = "/api/templates",
    params(TemplateSearchQuery),
    responses(
        (status = 200, description = "查询成功", body = ApiResponse<Vec<Template>>),
        (status = 500, description = "服务器错误")
    ),
    tag = "模板"
)]
pub async fn list_templates(
    State(state): State<AppState>,
    Query(params): Query<TemplateSearchQuery>,  // 从URL查询字符串提取参数
) -> Result<Json<Vec<Template>>, (StatusCode, String)> {
    // 从依赖注入容器获取模板服务
    let template_service = &state.module.template_service;
    
    // 提取分页参数，提供默认值
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);
    
    // 查询模板列表
    let templates = template_service
        .list_templates(page, page_size)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(templates))
}

/// 搜索模板
/// 
/// ## 端点
/// GET /api/templates/search?keyword=租房&location_tag=CN-BJ
/// 
/// ## 查询参数
/// - `keyword`: 搜索关键词（可选）- 在标题和描述中搜索
/// - `location_tag`: 地理标签（可选）- 如"CN"、"CN-BJ"、"CN-SH"
/// - `page`: 页码（可选，默认1）
/// - `page_size`: 每页数量（可选，默认20）
/// 
/// ## 认证
/// 无需认证（公开接口）
/// 
/// ## 响应
/// - 200 OK: 返回匹配的模板列表
/// - 500 Internal Server Error: 服务器错误
/// 
/// ## 搜索逻辑
/// 1. **关键词搜索**：在标题和描述中模糊匹配（ILIKE）
/// 2. **地理标签过滤**：精确匹配location_tag，同时包含通用模板（CN）
/// 3. **组合搜索**：可以同时使用关键词和地理标签
/// 
/// ## 示例
/// ```
/// # 搜索所有租房相关模板
/// GET /api/templates/search?keyword=租房
/// 
/// # 搜索北京的所有模板
/// GET /api/templates/search?location_tag=CN-BJ
/// 
/// # 搜索北京的租房模板
/// GET /api/templates/search?keyword=租房&location_tag=CN-BJ
/// ```
/// 
/// ## 地理标签说明
/// - `CN`: 全国通用模板
/// - `CN-BJ`: 北京专属模板
/// - `CN-SH`: 上海专属模板
/// - 搜索某城市时，会同时返回该城市和通用模板
/// 
/// ## 使用场景
/// - 用户搜索特定主题的模板
/// - 根据用户常驻城市推荐本地化模板
/// - 过滤特定地区的生活指南
#[utoipa::path(
    get,
    path = "/api/templates/search",
    params(TemplateSearchQuery),
    responses(
        (status = 200, description = "搜索成功", body = ApiResponse<Vec<Template>>),
        (status = 500, description = "服务器错误")
    ),
    tag = "模板"
)]
pub async fn search_templates(
    State(state): State<AppState>,
    Query(query): Query<TemplateSearchQuery>,
) -> Result<Json<Vec<Template>>, (StatusCode, String)> {
    // 从依赖注入容器获取模板服务
    let template_service = &state.module.template_service;
    
    // 执行搜索
    let templates = template_service
        .search_templates(query)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(templates))
}

/// 获取单个模板详情
/// 
/// ## 端点
/// GET /api/templates/:id
/// 
/// ## 路径参数
/// - `id`: 模板UUID
/// 
/// ## 认证
/// 无需认证（公开接口）
/// 
/// ## 响应
/// - 200 OK: 返回模板详情
/// - 404 Not Found: 模板不存在
/// 
/// ## 响应示例
/// ```json
/// {
///   "id": "uuid",
///   "title": "第一次在北京租房整租指南",
///   "description": "帮助你顺利完成在北京的第一次租房",
///   "location_tag": "CN-BJ",
///   "steps": [
///     {
///       "title": "确定预算和区域",
///       "description": "根据工作地点和收入确定租房预算",
///       "order": 0
///     },
///     {
///       "title": "寻找房源",
///       "description": "通过正规渠道寻找房源",
///       "order": 1
///     }
///   ],
///   "parent_id": null,
///   "created_by": "uuid",
///   "is_official": true,
///   "created_at": "2024-10-21T12:00:00Z",
///   "updated_at": "2024-10-21T12:00:00Z"
/// }
/// ```
/// 
/// ## 业务逻辑
/// 1. 根据模板ID查询数据库
/// 2. 返回完整的模板信息（包括所有步骤）
/// 
/// ## 使用场景
/// - 用户浏览模板详情
/// - Fork前预览模板内容
/// - 展示模板的所有步骤
#[utoipa::path(
    get,
    path = "/api/templates/{id}",
    params(
        ("id" = Uuid, Path, description = "模板UUID")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<Template>),
        (status = 404, description = "模板不存在"),
        (status = 500, description = "服务器错误")
    ),
    tag = "模板"
)]
pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,  // 从URL路径提取模板ID
) -> Result<Json<Template>, (StatusCode, String)> {
    // 从依赖注入容器获取模板服务
    let template_service = &state.module.template_service;
    
    // 查询模板详情
    let template = template_service
        .get_template(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(template))
}

/// 创建新模板
/// 
/// ## 端点
/// POST /api/templates
/// 
/// ## 认证
/// 需要JWT token
/// 
/// ## 请求体
/// ```json
/// {
///   "title": "第一次在上海找工作",
///   "description": "上海求职全流程指南",
///   "location_tag": "CN-SH",
///   "steps": [
///     {
///       "title": "准备简历",
///       "description": "制作一份专业的简历",
///       "order": 0
///     },
///     {
///       "title": "了解市场",
///       "description": "研究目标行业和公司",
///       "order": 1
///     }
///   ],
///   "parent_id": null  // 可选，父模板ID（用于模板继承）
/// }
/// ```
/// 
/// ## 响应
/// - 200 OK: 创建成功，返回新模板
/// - 400 Bad Request: 验证失败
/// - 401 Unauthorized: 未登录
/// 
/// ## 验证规则
/// - `title`: 1-200字符
/// - `description`: 1-2000字符
/// - `location_tag`: 有效的地理标签
/// - `steps`: 至少1个步骤
/// 
/// ## 业务逻辑
/// 1. 验证输入数据
/// 2. 记录创建者ID（从JWT token获取）
/// 3. 设置is_official=false（非官方模板）
/// 4. 保存到数据库
/// 5. 返回创建的模板
/// 
/// ## 权限说明
/// - V0.0.1版本：功能已实现但建议仅内部使用
/// - V0.1+版本：开放给所有用户创建模板
/// 
/// ## 使用场景
/// - 官方团队创建初始模板
/// - 种子用户贡献高质量模板
/// - 未来：普通用户创建和分享模板
/// 
/// ## 模板继承（高级功能）
/// 通过`parent_id`可以实现模板继承：
/// - 通用模板（CN）作为父模板
/// - 城市模板（CN-BJ）继承并扩展通用模板
/// - V0.0.1暂不实现继承逻辑，仅保留字段
#[utoipa::path(
    post,
    path = "/api/templates",
    request_body = CreateTemplateDto,
    responses(
        (status = 200, description = "创建成功", body = ApiResponse<Template>),
        (status = 400, description = "验证失败"),
        (status = 401, description = "未认证")
    ),
    security(("bearer_auth" = [])),
    tag = "模板"
)]
pub async fn create_template(
    State(state): State<AppState>,
    current_user: CurrentUser,  // JWT认证自动注入创建者ID
    Json(dto): Json<CreateTemplateDto>,
) -> Result<Json<Template>, (StatusCode, String)> {
    // 从依赖注入容器获取模板服务
    let template_service = &state.module.template_service;
    
    // 创建模板，记录创建者ID
    let template = template_service
        .create_template(dto, current_user.user_id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(template))
}

