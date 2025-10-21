use serde::Deserialize;

/// 应用程序总配置
/// 
/// 包含服务器、数据库、JWT等所有配置项。
/// 配置从环境变量或.env文件加载。
/// 
/// ## 配置来源优先级
/// 1. 系统环境变量（最高优先级）
/// 2. .env文件中的配置
/// 3. 代码中的默认值（最低优先级）
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// 服务器配置（监听地址、端口）
    pub server: ServerConfig,
    
    /// 数据库配置（连接URL、连接池大小）
    pub database: DatabaseConfig,
    
    /// JWT配置（密钥、过期时间）
    pub jwt: JwtConfig,
}

/// 服务器配置
/// 
/// 控制HTTP服务器的监听地址和端口
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// 监听主机地址（默认: 127.0.0.1）
    /// 
    /// - `127.0.0.1`: 仅本机访问
    /// - `0.0.0.0`: 允许外部访问
    pub host: String,
    
    /// 监听端口（默认: 8080）
    pub port: u16,
}

/// 数据库配置
/// 
/// 配置PostgreSQL连接参数
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库主机地址（默认: localhost）
    /// 
    /// 示例:
    /// - `localhost`: 本地数据库
    /// - `db`: Docker容器名称
    /// - `192.168.1.100`: 远程IP
    pub host: String,
    
    /// 数据库端口（默认: 5432）
    pub port: u16,
    
    /// 数据库用户名（默认: postgres）
    pub user: String,
    
    /// 数据库密码
    /// 
    /// **必需**：此配置项必须提供，否则应用启动失败
    pub password: String,
    
    /// 数据库名称（默认: rookie_guide）
    pub database_name: String,
    
    /// 最大数据库连接数（默认: 5）
    /// 
    /// 连接池大小影响并发性能：
    /// - 太小：高并发时性能瓶颈
    /// - 太大：占用过多数据库资源
    /// 
    /// 建议值：CPU核心数 * 2 到 CPU核心数 * 4
    pub max_connections: u32,
}

/// JWT配置
/// 
/// 配置JSON Web Token的生成和验证参数
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT签名密钥
    /// 
    /// **重要**: 生产环境必须使用强随机密钥！
    /// 
    /// 生成密钥示例:
    /// ```bash
    /// openssl rand -base64 64
    /// ```
    /// 
    /// **必需**：此配置项必须提供，否则应用启动失败
    pub secret: String,
    
    /// Token过期时间（秒）
    /// 
    /// 默认: 86400秒（24小时）
    /// 
    /// 常用值：
    /// - 3600: 1小时
    /// - 86400: 24小时
    /// - 604800: 7天
    pub expiration: i64,
}

impl DatabaseConfig {
    /// 构建PostgreSQL连接URL
    /// 
    /// 从分离的配置项（host, port, user, password, database_name）
    /// 构建完整的数据库连接URL字符串
    /// 
    /// ## 返回格式
    /// `postgresql://用户名:密码@主机:端口/数据库名`
    /// 
    /// ## 示例
    /// ```rust
    /// let url = db_config.connection_url();
    /// // postgresql://postgres:mypassword@localhost:5432/rookie_guide
    /// ```
    pub fn connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user,
            self.password,
            self.host,
            self.port,
            self.database_name
        )
    }
}

impl AppConfig {
    /// 从环境变量加载配置
    /// 
    /// ## 加载流程
    /// 1. 尝试加载当前目录的.env文件（如果存在）
    /// 2. 从环境变量读取各配置项
    /// 3. 对于可选配置，使用默认值
    /// 4. 对于必需配置（DATABASE_PASSWORD, JWT_SECRET），如果缺失则panic
    /// 
    /// ## 环境变量列表
    /// 
    /// ### 服务器配置
    /// - `SERVER_HOST`: 监听地址（默认: 127.0.0.1）
    /// - `SERVER_PORT`: 监听端口（默认: 8080）
    /// 
    /// ### 数据库配置
    /// - `DATABASE_HOST`: 数据库主机（默认: localhost）
    /// - `DATABASE_PORT`: 数据库端口（默认: 5432）
    /// - `DATABASE_USER`: 数据库用户名（默认: postgres）
    /// - `DATABASE_PASSWORD`: 数据库密码（**必需**）
    /// - `DATABASE_NAME`: 数据库名称（默认: rookie_guide）
    /// - `DATABASE_MAX_CONNECTIONS`: 最大连接数（默认: 5）
    /// 
    /// ### JWT配置
    /// - `JWT_SECRET`: JWT签名密钥（**必需**）
    /// - `JWT_EXPIRATION`: Token过期时间/秒（默认: 86400）
    /// 
    /// ## 错误处理
    /// 如果必需的配置项缺失，应用会panic并显示清晰的错误信息
    /// 
    /// ## 示例
    /// ```rust
    /// // 从环境变量加载配置
    /// let config = AppConfig::from_env()?;
    /// 
    /// // 使用配置
    /// println!("Server: {}:{}", config.server.host, config.server.port);
    /// println!("Database: {}", config.database.connection_url());
    /// ```
    pub fn from_env() -> anyhow::Result<Self> {
        // 尝试加载.env文件（如果存在）
        // ok()表示即使文件不存在也不报错
        dotenvy::dotenv().ok();
        
        Ok(AppConfig {
            server: ServerConfig {
                // SERVER_HOST环境变量，默认127.0.0.1（仅本机访问）
                host: std::env::var("SERVER_HOST")
                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                
                // SERVER_PORT环境变量，默认8080
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
            },
            database: DatabaseConfig {
                // DATABASE_HOST环境变量，默认localhost
                host: std::env::var("DATABASE_HOST")
                    .unwrap_or_else(|_| "localhost".to_string()),
                
                // DATABASE_PORT环境变量，默认5432
                port: std::env::var("DATABASE_PORT")
                    .unwrap_or_else(|_| "5432".to_string())
                    .parse()
                    .unwrap_or(5432),
                
                // DATABASE_USER环境变量，默认postgres
                user: std::env::var("DATABASE_USER")
                    .unwrap_or_else(|_| "postgres".to_string()),
                
                // DATABASE_PASSWORD环境变量（必需）
                // 如果未设置，应用将panic
                password: std::env::var("DATABASE_PASSWORD")
                    .expect("❌ DATABASE_PASSWORD环境变量未设置！请在.env文件中配置数据库密码"),
                
                // DATABASE_NAME环境变量，默认rookie_guide
                database_name: std::env::var("DATABASE_NAME")
                    .unwrap_or_else(|_| "rookie_guide".to_string()),
                
                // DATABASE_MAX_CONNECTIONS环境变量，默认5
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
            },
            jwt: JwtConfig {
                // JWT_SECRET环境变量（必需）
                // 如果未设置，应用将panic
                secret: std::env::var("JWT_SECRET")
                    .expect("❌ JWT_SECRET环境变量未设置！请在.env文件中配置JWT密钥"),
                
                // JWT_EXPIRATION环境变量，默认86400秒（24小时）
                expiration: std::env::var("JWT_EXPIRATION")
                    .unwrap_or_else(|_| "86400".to_string())
                    .parse()
                    .unwrap_or(86400),
            },
        })
    }
}

