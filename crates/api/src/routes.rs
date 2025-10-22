use crate::{handlers, state::AppState};
use axum::{
    routing::{get, post, put},
    Router,
};

/// 创建应用程序路由
/// 
/// 该函数定义了所有HTTP端点的路由规则，包括：
/// - 健康检查
/// - 用户认证（注册/登录）
/// - 用户管理
/// - 模板管理
/// - 清单管理
/// 
/// ## 路由分组
/// - `/health` - 健康检查，用于监控服务状态
/// - `/api/auth/*` - 认证相关，无需token
/// - `/api/users/*` - 用户管理，需要token
/// - `/api/templates/*` - 模板管理，部分需要token
/// - `/api/checklists/*` - 清单管理，需要token
/// 
/// ## 参数
/// * `state` - 应用状态，包含依赖注入容器
/// 
/// ## 返回
/// 返回配置好的Axum路由器
pub fn create_router(state: AppState) -> Router {
    // 先创建文档路由（无状态），然后再创建业务路由（有状态）
    let api_routes = Router::new()
        // ==================== 健康检查 ====================
        // GET /health - 返回服务状态，用于健康检查和监控
        .route("/health", get(handlers::health::health_check))
        
        // ==================== 认证路由（公开） ====================
        // POST /api/auth/register - 用户注册
        .route("/api/auth/register", post(handlers::auth::register))
        // POST /api/auth/login - 用户登录
        .route("/api/auth/login", post(handlers::auth::login))
        
        // ==================== 用户路由（需要认证） ====================
        // GET /api/users/me - 获取当前登录用户信息
        .route("/api/users/me", get(handlers::user::get_current_user))
        // PUT /api/users/me - 更新当前用户资料
        .route("/api/users/me", put(handlers::user::update_profile))
        
        // ==================== 模板路由 ====================
        // GET /api/templates - 列出所有模板（分页）
        .route("/api/templates", get(handlers::template::list_templates))
        // GET /api/templates/search - 搜索模板（支持关键词、地理标签）
        .route("/api/templates/search", get(handlers::template::search_templates))
        // GET /api/templates/:id - 获取单个模板详情
        .route("/api/templates/:id", get(handlers::template::get_template))
        // POST /api/templates - 创建新模板（需要认证）
        .route("/api/templates", post(handlers::template::create_template))
        
        // ==================== 清单路由（需要认证） ====================
        // GET /api/checklists - 获取当前用户的所有清单
        .route("/api/checklists", get(handlers::checklist::get_user_checklists))
        // POST /api/checklists - Fork模板到个人清单
        .route("/api/checklists", post(handlers::checklist::fork_template))
        // GET /api/checklists/:id - 获取单个清单详情
        .route("/api/checklists/:id", get(handlers::checklist::get_checklist))
        // PUT /api/checklists/:id/steps - 更新清单中某个步骤的完成状态
        .route("/api/checklists/:id/steps", put(handlers::checklist::update_step))
        
        // 注入应用状态，使所有handler都能访问服务
        .with_state(state);
    
    // 合并文档路由和业务路由
    crate::docs::docs_routes().merge(api_routes)
}

