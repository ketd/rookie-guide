/// OpenAPI 文档路由模块
/// 
/// 使用 utoipa 生成 OpenAPI 3.0 规范文档，提供 Swagger UI 和 ReDoc 界面

use utoipa::OpenApi;
use axum::{Router, routing::get, response::Html};

// 导入所有模型以便在文档中使用
use models::{
    // 用户相关
    User, UserProfile, RegisterDto, LoginDto, UpdateProfileDto, AuthResponse,
    // 模板相关
    Template, TemplateStep, LocationTag, CreateTemplateDto, UpdateTemplateDto, TemplateSearchQuery,
    // 清单相关
    UserChecklist, StepProgress, ChecklistProgress, ForkTemplateDto, UpdateStepDto, UserChecklistResponse,
};

// 导入 ApiResponse 用于文档
use common::ApiResponse;

/// 主 OpenAPI 文档定义
/// 
/// 聚合所有模块的 API 文档到一个统一的 OpenAPI 规范中
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rookie Guide API",
        version = "0.0.1",
        description = "新手村 - 第一次体验指南 API 文档\n\n提供用户认证、经验模板管理、个人清单进度追踪等功能。",
        contact(
            name = "Rookie Guide Team",
            email = "support@rookieguide.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    // 定义所有要文档化的路径（handler 函数）
    paths(
        // 健康检查
        crate::handlers::health::health_check,
        
        // 认证相关
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        
        // 用户相关
        crate::handlers::user::get_current_user,
        crate::handlers::user::update_profile,
        
        // 模板相关
        crate::handlers::template::list_templates,
        crate::handlers::template::search_templates,
        crate::handlers::template::get_template,
        crate::handlers::template::create_template,
        
        // 清单相关
        crate::handlers::checklist::get_user_checklists,
        crate::handlers::checklist::fork_template,
        crate::handlers::checklist::get_checklist,
        crate::handlers::checklist::update_step,
    ),
    // 定义所有要文档化的组件（数据模型）
    components(schemas(
        // 通用响应
        ApiResponse<UserProfile>,
        ApiResponse<AuthResponse>,
        ApiResponse<Template>,
        ApiResponse<Vec<Template>>,
        ApiResponse<UserChecklistResponse>,
        ApiResponse<Vec<UserChecklistResponse>>,
        
        // 用户模型
        User,
        UserProfile,
        RegisterDto,
        LoginDto,
        UpdateProfileDto,
        AuthResponse,
        
        // 模板模型
        Template,
        TemplateStep,
        LocationTag,
        CreateTemplateDto,
        UpdateTemplateDto,
        TemplateSearchQuery,
        
        // 清单模型
        UserChecklist,
        StepProgress,
        ChecklistProgress,
        ForkTemplateDto,
        UpdateStepDto,
        UserChecklistResponse,
    )),
    // 定义标签（用于API分组）
    tags(
        (name = "健康检查", description = "服务健康状态检查"),
        (name = "认证", description = "用户注册、登录相关接口"),
        (name = "用户", description = "用户资料管理"),
        (name = "模板", description = "经验模板浏览、创建"),
        (name = "清单", description = "个人清单管理、进度追踪"),
    ),
    // 定义安全方案（JWT 认证）
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// 安全方案配置（JWT Bearer Token）
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("请输入JWT令牌（无需添加 'Bearer ' 前缀）"))
                        .build(),
                ),
            );
        }
    }
}

/// 创建文档路由
/// 
/// ## 可访问的文档页面
/// 
/// - `/docs` - 文档首页（选择 Swagger UI 或 ReDoc）
/// - `/docs/swagger-ui` - Swagger UI 交互式文档
/// - `/docs/redoc` - ReDoc 文档（更适合阅读）
/// - `/api-docs/openapi.json` - OpenAPI JSON 规范文件
pub fn docs_routes() -> Router {
    // 创建 OpenAPI 规范
    let openapi = ApiDoc::openapi();
    
    Router::new()
        // 文档首页
        .route("/docs", get(docs_index))
        // OpenAPI JSON
        .route("/api-docs/openapi.json", get(move || async move { 
            axum::Json(openapi)
        }))
}

/// 文档首页 HTML
/// 
/// 提供友好的导航页面，引导用户选择不同的文档查看方式
async fn docs_index() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rookie Guide API 文档</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }
        .container {
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            max-width: 600px;
            width: 100%;
            padding: 40px;
        }
        h1 {
            color: #333;
            font-size: 32px;
            margin-bottom: 10px;
            text-align: center;
        }
        .subtitle {
            color: #666;
            text-align: center;
            margin-bottom: 30px;
            font-size: 14px;
        }
        .description {
            color: #555;
            line-height: 1.8;
            margin-bottom: 30px;
            text-align: center;
        }
        .links {
            display: flex;
            flex-direction: column;
            gap: 15px;
        }
        .links a {
            display: flex;
            align-items: center;
            padding: 18px 24px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            text-decoration: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 500;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        .links a:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 20px rgba(102, 126, 234, 0.4);
        }
        .links a .icon {
            font-size: 24px;
            margin-right: 12px;
        }
        .links a .description {
            font-size: 12px;
            opacity: 0.9;
            margin-top: 4px;
            text-align: left;
        }
        .footer {
            margin-top: 30px;
            text-align: center;
            color: #999;
            font-size: 12px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎯 Rookie Guide API</h1>
        <div class="subtitle">v0.0.1 | OpenAPI 3.0 规范</div>
        <div class="description">
            欢迎使用 Rookie Guide API 文档！<br>
            请选择您喜欢的文档查看方式：
        </div>
        <div class="links">
            <a href="/docs/swagger-ui">
                <span class="icon">📋</span>
                <div>
                    <div>Swagger UI</div>
                    <div class="description">交互式API文档，可直接测试接口</div>
                </div>
            </a>
            <a href="/docs/redoc">
                <span class="icon">📘</span>
                <div>
                    <div>ReDoc</div>
                    <div class="description">优雅的阅读式文档，适合深入了解</div>
                </div>
            </a>
            <a href="/api-docs/openapi.json">
                <span class="icon">📄</span>
                <div>
                    <div>OpenAPI JSON</div>
                    <div class="description">原始 OpenAPI 规范文件</div>
                </div>
            </a>
        </div>
        <div class="footer">
            Powered by Axum + utoipa | MIT License
        </div>
    </div>
</body>
</html>
    "#)
}

