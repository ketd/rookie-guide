use common::AppConfig;
use service_layer::AppModule;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// 应用程序全局状态
/// 
/// 该结构体包含了应用运行所需的所有共享状态，主要是依赖注入容器。
/// 通过Axum的State机制，可以在所有HTTP处理器中访问这些服务。
/// 
/// ## Clone语义
/// AppState实现了Clone，但这是浅拷贝（Arc的引用计数增加），
/// 不会复制实际的服务实例，保证了整个应用共享同一套服务。
#[derive(Clone)]
pub struct AppState {
    /// 依赖注入容器，包含所有业务服务
    /// 使用Arc包装以实现跨请求共享和线程安全
    pub module: Arc<AppModule>,
}

impl AppState {
    /// 创建新的应用状态
    /// 
    /// ## 参数
    /// * `db` - SeaORM 数据库连接
    /// * `config` - 应用配置
    /// 
    /// ## 返回
    /// 返回包含完整依赖注入容器的应用状态
    pub fn new(db: DatabaseConnection, config: AppConfig) -> Self {
        // 初始化依赖注入容器
        let module = AppModule::new(db, config);
        
        Self {
            // 使用Arc包装，允许在多个请求之间共享
            module: Arc::new(module),
        }
    }
}

