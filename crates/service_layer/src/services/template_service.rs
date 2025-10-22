use async_trait::async_trait;
use common::{AppResult, AppError};
use models::{Template, CreateTemplateDto, TemplateSearchQuery};
use db::TemplateRepository;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[async_trait]
pub trait TemplateService: Send + Sync {
    async fn create_template(&self, dto: CreateTemplateDto, created_by: Uuid) -> AppResult<Template>;
    async fn get_template(&self, id: Uuid) -> AppResult<Template>;
    async fn search_templates(&self, query: TemplateSearchQuery) -> AppResult<Vec<Template>>;
    async fn get_templates_by_city(&self, city: String) -> AppResult<Vec<Template>>;
    async fn list_templates(&self, page: i32, page_size: i32) -> AppResult<Vec<Template>>;
}

pub struct TemplateServiceImpl {
    template_repo: Arc<dyn TemplateRepository>,
}

impl TemplateServiceImpl {
    pub fn new(template_repo: Arc<dyn TemplateRepository>) -> Self {
        Self { template_repo }
    }
}

#[async_trait]
impl TemplateService for TemplateServiceImpl {
    async fn create_template(&self, dto: CreateTemplateDto, created_by: Uuid) -> AppResult<Template> {
        // Validate input
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Create template
        self.template_repo.create(dto, created_by).await
    }

    async fn get_template(&self, id: Uuid) -> AppResult<Template> {
        self.template_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Template {} not found", id)))
    }

    async fn search_templates(&self, query: TemplateSearchQuery) -> AppResult<Vec<Template>> {
        self.template_repo.search(query).await
    }

    async fn get_templates_by_city(&self, city: String) -> AppResult<Vec<Template>> {
        self.template_repo.find_by_location(city).await
    }

    async fn list_templates(&self, page: i32, page_size: i32) -> AppResult<Vec<Template>> {
        self.template_repo.list_all(page, page_size).await
    }
}

