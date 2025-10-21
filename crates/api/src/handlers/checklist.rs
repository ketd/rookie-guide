use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use models::{UserChecklistResponse, ForkTemplateDto, UpdateStepDto};
use common::ApiResponse;
use crate::{middleware::CurrentUser, state::AppState};
use uuid::Uuid;

/// 获取当前用户的所有清单
/// 
/// ## 端点
/// GET /api/checklists
/// 
/// ## 认证
/// 需要JWT token（通过CurrentUser中间件）
/// 
/// ## 响应
/// - 200 OK: 返回用户的所有清单列表（包含进度信息）
/// - 500 Internal Server Error: 服务器错误
/// 
/// ## 响应示例
/// ```json
/// [
///   {
///     "checklist": {
///       "id": "uuid",
///       "user_id": "uuid",
///       "source_template_id": "uuid",
///       "title": "第一次在北京租房",
///       "progress_status": [...],
///       "created_at": "2024-10-21T12:00:00Z",
///       "updated_at": "2024-10-21T12:00:00Z"
///     },
///     "progress": {
///       "total_steps": 10,
///       "completed_steps": 3,
///       "progress_percentage": 30.0
///     }
///   }
/// ]
/// ```
/// 
/// ## 业务逻辑
/// 1. 从JWT token提取当前用户ID
/// 2. 查询该用户的所有清单
/// 3. 计算每个清单的完成进度
/// 4. 返回清单列表和进度信息
#[utoipa::path(
    get,
    path = "/api/checklists",
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<Vec<UserChecklistResponse>>),
        (status = 401, description = "未认证"),
        (status = 500, description = "服务器错误")
    ),
    security(("bearer_auth" = [])),
    tag = "清单"
)]
pub async fn get_user_checklists(
    State(state): State<AppState>,
    current_user: CurrentUser,  // JWT认证自动注入
) -> Result<Json<Vec<UserChecklistResponse>>, (StatusCode, String)> {
    // 从依赖注入容器获取清单服务
    let checklist_service = &state.module.checklist_service;
    
    // 查询当前用户的所有清单
    let checklists = checklist_service
        .get_user_checklists(current_user.user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(checklists))
}

/// Fork模板到个人清单
/// 
/// ## 端点
/// POST /api/checklists
/// 
/// ## 认证
/// 需要JWT token
/// 
/// ## 请求体
/// ```json
/// {
///   "template_id": "uuid"  // 要Fork的模板ID
/// }
/// ```
/// 
/// ## 响应
/// - 200 OK: Fork成功，返回新创建的清单
/// - 400 Bad Request: 模板不存在或参数错误
/// - 401 Unauthorized: 未登录
/// 
/// ## 业务逻辑
/// 1. 验证模板是否存在
/// 2. 复制模板的标题和步骤到用户清单
/// 3. 初始化所有步骤为未完成状态
/// 4. 创建清单记录
/// 5. 返回新清单和初始进度（0%）
/// 
/// ## 注意事项
/// - V0.0.1版本：Fork后的清单不可修改步骤
/// - 同一模板可以被同一用户多次Fork
/// - Fork的是模板的快照，后续模板修改不影响已Fork的清单
#[utoipa::path(
    post,
    path = "/api/checklists",
    request_body = ForkTemplateDto,
    responses(
        (status = 200, description = "Fork成功", body = ApiResponse<UserChecklistResponse>),
        (status = 400, description = "模板不存在"),
        (status = 401, description = "未认证")
    ),
    security(("bearer_auth" = [])),
    tag = "清单"
)]
pub async fn fork_template(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(dto): Json<ForkTemplateDto>,
) -> Result<Json<UserChecklistResponse>, (StatusCode, String)> {
    // 从依赖注入容器获取清单服务
    let checklist_service = &state.module.checklist_service;
    
    // 执行Fork操作
    let checklist = checklist_service
        .fork_template(current_user.user_id, dto)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(checklist))
}

/// 获取单个清单详情
/// 
/// ## 端点
/// GET /api/checklists/:id
/// 
/// ## 路径参数
/// - `id`: 清单UUID
/// 
/// ## 响应
/// - 200 OK: 返回清单详情和进度
/// - 404 Not Found: 清单不存在
/// 
/// ## 响应示例
/// ```json
/// {
///   "checklist": {
///     "id": "uuid",
///     "title": "第一次在北京租房",
///     "progress_status": [
///       {
///         "step_index": 0,
///         "completed": true,
///         "completed_at": "2024-10-21T12:00:00Z"
///       }
///     ]
///   },
///   "progress": {
///     "total_steps": 10,
///     "completed_steps": 1,
///     "progress_percentage": 10.0
///   }
/// }
/// ```
/// 
/// ## 业务逻辑
/// 1. 根据清单ID查询数据库
/// 2. 计算当前完成进度
/// 3. 返回清单详情和进度统计
/// 
/// ## 权限说明
/// - V0.0.1版本：任何人都可以查看任何清单
/// - TODO V0.1+：只能查看自己的清单或公开分享的清单
#[utoipa::path(
    get,
    path = "/api/checklists/{id}",
    params(
        ("id" = Uuid, Path, description = "清单UUID")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<UserChecklistResponse>),
        (status = 404, description = "清单不存在"),
        (status = 500, description = "服务器错误")
    ),
    tag = "清单"
)]
pub async fn get_checklist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,  // 从URL路径提取清单ID
) -> Result<Json<UserChecklistResponse>, (StatusCode, String)> {
    // 从依赖注入容器获取清单服务
    let checklist_service = &state.module.checklist_service;
    
    // 查询清单详情
    let checklist = checklist_service
        .get_checklist(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(checklist))
}

/// 更新清单中某个步骤的完成状态
/// 
/// ## 端点
/// PUT /api/checklists/:id/steps
/// 
/// ## 路径参数
/// - `id`: 清单UUID
/// 
/// ## 请求体
/// ```json
/// {
///   "step_index": 0,      // 步骤索引（从0开始）
///   "completed": true     // 完成状态：true=已完成，false=未完成
/// }
/// ```
/// 
/// ## 响应
/// - 200 OK: 更新成功，返回更新后的清单和进度
/// - 400 Bad Request: 步骤索引无效或参数错误
/// 
/// ## 业务逻辑
/// 1. 查找指定的清单
/// 2. 更新指定步骤的完成状态
/// 3. 如果标记为完成，记录完成时间
/// 4. 重新计算整体进度
/// 5. 返回更新后的清单
/// 
/// ## 核心功能
/// 这是"进度追踪"的核心功能，用户通过勾选步骤来：
/// - ✅ 记录自己的进展
/// - 📊 看到可视化的完成度
/// - 🎯 保持行动的动力
/// 
/// ## 示例场景
/// 用户完成了"第一次租房"清单中的"确定预算"步骤：
/// ```
/// PUT /api/checklists/{id}/steps
/// { "step_index": 0, "completed": true }
/// 
/// → 进度从 0% 更新为 10%（假设共10步）
/// → completed_at 记录为当前时间
/// ```
#[utoipa::path(
    put,
    path = "/api/checklists/{id}/steps",
    params(
        ("id" = Uuid, Path, description = "清单UUID")
    ),
    request_body = UpdateStepDto,
    responses(
        (status = 200, description = "更新成功", body = ApiResponse<UserChecklistResponse>),
        (status = 400, description = "步骤索引无效"),
        (status = 404, description = "清单不存在")
    ),
    tag = "清单"
)]
pub async fn update_step(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,  // 从URL路径提取清单ID
    Json(dto): Json<UpdateStepDto>,
) -> Result<Json<UserChecklistResponse>, (StatusCode, String)> {
    // 从依赖注入容器获取清单服务
    let checklist_service = &state.module.checklist_service;
    
    // 更新步骤状态
    let checklist = checklist_service
        .update_step(id, dto)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(checklist))
}

