use auth::{JwtService, JwtServiceImpl, PasswordService, PasswordServiceImpl};
use common::AppConfig;
use db::{
    TemplateRepository, TemplateRepositoryImpl,
    UserRepository, UserRepositoryImpl,
    UserChecklistRepository, UserChecklistRepositoryImpl,
};
use crate::services::{
    TemplateService, TemplateServiceImpl,
    UserService, UserServiceImpl,
    ChecklistService, ChecklistServiceImpl,
};
use sqlx::PgPool;
use std::sync::Arc;

/// Application Dependency Injection Container
/// 
/// This module implements a manual dependency injection pattern using trait objects
/// and Arc for thread-safe shared ownership. All dependencies are constructed in the
/// correct order and injected through constructors.
/// 
/// ## Architecture:
/// ```
/// AppModule
///   ├── TemplateService (depends on TemplateRepository)
///   ├── UserService (depends on UserRepository, JwtService, PasswordService)
///   └── ChecklistService (depends on UserChecklistRepository, TemplateRepository)
/// ```
pub struct AppModule {
    pub template_service: Arc<dyn TemplateService>,
    pub user_service: Arc<dyn UserService>,
    pub checklist_service: Arc<dyn ChecklistService>,
}

impl AppModule {
    /// Create a new AppModule with all dependencies properly wired up
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `config` - Application configuration
    /// 
    /// # Returns
    /// A fully initialized AppModule with all services ready to use
    pub fn new(pool: PgPool, config: AppConfig) -> Self {
        // Layer 1: Create repository instances (data access layer)
        let template_repo = Arc::new(TemplateRepositoryImpl::new(pool.clone())) as Arc<dyn TemplateRepository>;
        let user_repo = Arc::new(UserRepositoryImpl::new(pool.clone())) as Arc<dyn UserRepository>;
        let checklist_repo = Arc::new(UserChecklistRepositoryImpl::new(pool.clone())) as Arc<dyn UserChecklistRepository>;

        // Layer 2: Create auth service instances (infrastructure layer)
        let jwt_service = Arc::new(JwtServiceImpl::new(
            config.jwt.secret.clone(),
            config.jwt.expiration,
        )) as Arc<dyn JwtService>;
        
        let password_service = Arc::new(PasswordServiceImpl::new()) as Arc<dyn PasswordService>;

        // Layer 3: Create business service instances (business logic layer)
        // Services are injected with their dependencies from layers 1 and 2
        let template_service = Arc::new(TemplateServiceImpl::new(
            template_repo.clone()
        )) as Arc<dyn TemplateService>;
        
        let user_service = Arc::new(UserServiceImpl::new(
            user_repo.clone(),
            jwt_service.clone(),
            password_service.clone(),
        )) as Arc<dyn UserService>;
        
        let checklist_service = Arc::new(ChecklistServiceImpl::new(
            checklist_repo.clone(),
            template_repo.clone(),
        )) as Arc<dyn ChecklistService>;

        Self {
            template_service,
            user_service,
            checklist_service,
        }
    }
}

