use sea_orm::{Database, DatabaseConnection, DbErr, ConnectOptions};
use std::time::Duration;

/// 创建SeaORM数据库连接
/// 
/// 连接池是数据库访问的核心组件，负责：
/// - **自动创建数据库**（如果不存在）
/// - 复用数据库连接，避免频繁创建/销毁
/// - 限制并发连接数，保护数据库
/// - 处理连接超时和重连
/// 
/// ## 参数
/// 
/// - `database_url`: 数据库连接URL
///   - 格式: `postgresql://用户名:密码@主机:端口/数据库名`
///   - 示例: `postgresql://postgres:password@localhost:5432/rookie_guide`
///   - 通常由 `DatabaseConfig::connection_url()` 生成
/// 
/// - `max_connections`: 连接池最大连接数
///   - 默认值: 5（配置中的DATABASE_MAX_CONNECTIONS）
///   - 建议值: CPU核心数 × 2 到 CPU核心数 × 4
///   - 过小: 高并发时性能瓶颈
///   - 过大: 占用过多数据库资源
/// 
/// ## 返回值
/// 
/// - `Ok(DatabaseConnection)`: 成功创建的数据库连接
/// - `Err(DbErr)`: 连接失败（如数据库不可达、密码错误等）
/// 
/// ## 配置
/// 
/// - **最大连接数**: 由参数指定
/// - **连接超时**: 30秒
/// - **获取超时**: 30秒
/// - **空闲超时**: 8秒
/// - **最大生命周期**: 3600秒（1小时）
/// 
/// ## 使用示例
/// 
/// ```rust
/// // 创建数据库连接
/// let db = create_database_connection(
///     "postgresql://postgres:password@localhost:5432/rookie_guide",
///     5  // 最大5个连接
/// ).await?;
/// 
/// // 使用 SeaORM 查询
/// use models::UserEntity;
/// let users = UserEntity::find()
///     .all(&db)
///     .await?;
/// ```
/// 
/// ## 错误处理
/// 
/// 如果连接失败，常见原因：
/// - ❌ PostgreSQL服务未启动：`docker-compose up -d` 或启动本地PostgreSQL
/// - ❌ 配置错误：检查DATABASE_HOST、DATABASE_PORT、DATABASE_PASSWORD等配置
/// - ❌ 密码错误：确认数据库凭据
/// - ❌ 权限不足：确保用户有创建数据库的权限
/// - ❌ 网络不通：检查防火墙和网络连接
/// 
/// **注意**: 数据库会自动创建，无需手动执行 `createdb`
/// 
/// ## 生命周期
/// 
/// ```
/// 应用启动
///     ↓
/// create_database_connection()  ← 创建连接池（本函数）
///     ↓
/// 应用运行期间：连接池自动管理连接
///     ↓
/// 应用关闭：连接池自动清理
/// ```
pub async fn create_database_connection(
    database_url: &str, 
    max_connections: u32
) -> Result<DatabaseConnection, DbErr> {
    // 确保数据库存在（如果不存在则创建）
    ensure_database_exists(database_url).await?;
    
    // 配置 SeaORM 连接选项
    let mut opt = ConnectOptions::new(database_url.to_owned());
    opt.max_connections(max_connections)
        .connect_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(3600))
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Debug);
    
    // 连接到数据库
    Database::connect(opt).await
}

/// 确保数据库存在，如果不存在则自动创建
/// 
/// ## 工作流程
/// 1. 尝试连接到目标数据库
/// 2. 如果失败（数据库不存在），连接到 postgres 默认数据库
/// 3. 执行 CREATE DATABASE 命令
/// 4. 返回成功
/// 
/// ## 参数
/// - `database_url`: 目标数据库连接URL
/// 
/// ## 使用示例
/// ```rust
/// ensure_database_exists(&database_url).await?;
/// ```
async fn ensure_database_exists(database_url: &str) -> Result<(), DbErr> {
    // 解析数据库URL以获取数据库名
    let database_name = extract_database_name(database_url);
    
    // 尝试连接到目标数据库
    let test_result = Database::connect(database_url).await;
    
    if test_result.is_ok() {
        // 数据库已存在
        tracing::info!("数据库 '{}' 已存在", database_name);
        return Ok(());
    }
    
    // 数据库不存在，尝试创建
    tracing::info!("数据库 '{}' 不存在，正在创建...", database_name);
    
    // 构建连接到 postgres 默认数据库的URL
    let postgres_url = database_url.replace(&format!("/{}", database_name), "/postgres");
    
    // 连接到 postgres 数据库
    let db = Database::connect(&postgres_url).await.map_err(|e| {
        DbErr::Custom(format!(
            "无法连接到 PostgreSQL 服务器：{}。请确保 PostgreSQL 正在运行（使用 docker-compose up -d）",
            e
        ))
    })?;
    
    // 执行 CREATE DATABASE 命令
    let create_db_sql = format!("CREATE DATABASE \"{}\"", database_name);
    
    // 使用底层连接执行原生 SQL
    use sea_orm::ConnectionTrait;
    db.execute_unprepared(&create_db_sql).await.map_err(|e| {
        DbErr::Custom(format!("创建数据库失败：{}", e))
    })?;
    
    tracing::info!("✅ 数据库 '{}' 创建成功", database_name);
    
    Ok(())
}

/// 从数据库URL中提取数据库名
/// 
/// ## 示例
/// ```
/// postgresql://user:pass@host:5432/mydb -> "mydb"
/// ```
fn extract_database_name(database_url: &str) -> &str {
    database_url
        .rsplit('/')
        .next()
        .unwrap_or("rookie_guide")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_database_name() {
        assert_eq!(
            extract_database_name("postgresql://postgres:password@localhost:5432/rookie_guide"),
            "rookie_guide"
        );
        assert_eq!(
            extract_database_name("postgresql://user:pass@host/testdb"),
            "testdb"
        );
    }
}
