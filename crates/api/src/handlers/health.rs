use axum::Json;
use serde_json::{json, Value};
use utoipa;

/// 健康检查端点
/// 
/// ## 端点
/// GET /health
/// 
/// ## 认证
/// 无需认证（公开接口）
/// 
/// ## 响应
/// 始终返回200 OK（除非服务完全down）
/// 
/// ## 响应示例
/// ```json
/// {
///   "status": "ok",
///   "service": "rookie-guide-api",
///   "version": "0.0.1"
/// }
/// ```
/// 
/// ## 用途
/// 1. **容器健康检查**: Docker/K8s使用此端点判断服务是否健康
/// 2. **监控告警**: 监控系统定期访问此端点检测服务可用性
/// 3. **负载均衡**: 负载均衡器通过此端点判断是否将流量分发到此实例
/// 4. **调试验证**: 快速验证服务是否正常启动
/// 
/// ## 示例用法
/// ```bash
/// # 检查服务是否启动
/// curl http://localhost:8080/health
/// 
/// # Docker健康检查
/// HEALTHCHECK --interval=30s --timeout=3s \
///   CMD curl -f http://localhost:8080/health || exit 1
/// 
/// # K8s健康探针
/// livenessProbe:
///   httpGet:
///     path: /health
///     port: 8080
///   initialDelaySeconds: 10
///   periodSeconds: 30
/// ```
/// 
/// ## 注意事项
/// - 这是一个简单的健康检查，只验证HTTP服务是否响应
/// - V0.1+可以扩展为检查数据库连接、依赖服务等
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "服务正常", body = serde_json::Value, 
         example = json!({"status": "ok", "service": "rookie-guide-api", "version": "0.0.1"}))
    ),
    tag = "健康检查"
)]
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "rookie-guide-api",
        "version": "0.0.1"
    }))
}

