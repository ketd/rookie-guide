mod handlers;
mod middleware;
mod routes;
mod state;
mod docs;

use common::AppConfig;
use db::create_database_connection;
use migration::{Migrator, MigratorTrait};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// 应用程序主入口
/// 
/// 启动流程：
/// 1. 初始化日志系统
/// 2. 加载配置文件
/// 3. 创建数据库连接（SeaORM）
/// 4. **强制运行数据库迁移（SeaORM Migration，确保数据库结构最新）**
/// 5. 初始化依赖注入容器
/// 6. 构建路由和中间件
/// 7. 启动HTTP服务器
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ==================== 1. 初始化日志系统 ====================
    // 设置日志级别，可通过环境变量RUST_LOG控制
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug,sea_orm=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ==================== 2. 加载配置 ====================
    // 从环境变量或.env文件加载配置
    let config = AppConfig::from_env()?;
    
    tracing::info!("🚀 启动阅历进度条 API 服务器...");
    tracing::info!("📊 数据库连接: {}@{}:{}/{}", 
        config.database.user,
        config.database.host,
        config.database.port,
        config.database.database_name
    );

    // ==================== 3. 创建数据库连接 ====================
    let database_url = config.database.connection_url();
    
    tracing::info!("🌊 创建 SeaORM 数据库连接...");
    let db = create_database_connection(&database_url, config.database.max_connections).await
        .map_err(|e| {
            tracing::error!("❌ 数据库连接失败: {}", e);
            anyhow::anyhow!(e)
        })?;
    
    tracing::info!("✅ 数据库连接成功");

    // ==================== 4. 强制运行数据库迁移 ====================
    // 【数据库先行】每次启动时自动同步数据库结构
    // 使用 SeaORM Migration（Rust 代码定义迁移，类型安全）
    tracing::info!("🔄 开始同步数据库结构（使用 SeaORM Migration）...");
    
    Migrator::up(&db, None).await
        .map_err(|e| {
            tracing::error!("❌ 数据库迁移失败: {}", e);
            tracing::error!("💡 提示：请检查 migration crate 中的迁移定义是否正确");
            anyhow::anyhow!(e)
        })?;
    
    tracing::info!("✅ 数据库结构同步完成");

    // ==================== 5. 初始化依赖注入容器 ====================
    // 创建应用状态，包含所有服务的依赖注入容器
    let app_state = state::AppState::new(db, config.clone());
    tracing::info!("✅ 依赖注入容器初始化完成");

    // ==================== 6. 构建路由和中间件 ====================
    // 配置HTTP路由、CORS跨域、请求追踪等中间件
    let app = routes::create_router(app_state)
        .layer(CorsLayer::permissive())  // 允许跨域请求
        .layer(tower_http::trace::TraceLayer::new_for_http());  // HTTP请求追踪

    // ==================== 7. 启动HTTP服务器 ====================
    let addr = SocketAddr::from(([127, 0, 0, 1], config.server.port));
    tracing::info!("🌐 服务器监听地址: http://{}", addr);
    tracing::info!("📖 健康检查: http://{}/health", addr);
    tracing::info!("📚 API 文档: http://{}/docs", addr);
    tracing::info!("📋 Swagger UI: http://{}/docs/swagger-ui", addr);
    tracing::info!("📘 ReDoc: http://{}/docs/redoc", addr);
    tracing::info!("🎉 服务器启动成功！");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
