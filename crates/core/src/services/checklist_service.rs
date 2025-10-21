use async_trait::async_trait;
use common::{AppResult, AppError};
use models::{UserChecklistResponse, ForkTemplateDto, UpdateStepDto};
use db::{UserChecklistRepository, TemplateRepository};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait ChecklistService: Send + Sync {
    async fn fork_template(&self, user_id: Uuid, dto: ForkTemplateDto) -> AppResult<UserChecklistResponse>;
    async fn get_checklist(&self, checklist_id: Uuid) -> AppResult<UserChecklistResponse>;
    async fn get_user_checklists(&self, user_id: Uuid) -> AppResult<Vec<UserChecklistResponse>>;
    async fn update_step(&self, checklist_id: Uuid, dto: UpdateStepDto) -> AppResult<UserChecklistResponse>;
}

pub struct ChecklistServiceImpl {
    checklist_repo: Arc<dyn UserChecklistRepository>,
    template_repo: Arc<dyn TemplateRepository>,
}

impl ChecklistServiceImpl {
    pub fn new(
        checklist_repo: Arc<dyn UserChecklistRepository>,
        template_repo: Arc<dyn TemplateRepository>,
    ) -> Self {
        Self {
            checklist_repo,
            template_repo,
        }
    }
}

#[async_trait]
impl ChecklistService for ChecklistServiceImpl {
    async fn fork_template(&self, user_id: Uuid, dto: ForkTemplateDto) -> AppResult<UserChecklistResponse> {
        // Get the template
        let template = self.template_repo
            .find_by_id(dto.template_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Template {} not found", dto.template_id)))?;

        // Create checklist from template
        let checklist = self.checklist_repo
            .create_from_template(user_id, &template)
            .await?;

        // Calculate progress
        let progress = checklist.calculate_progress();

        Ok(UserChecklistResponse {
            checklist,
            progress,
        })
    }

    async fn get_checklist(&self, checklist_id: Uuid) -> AppResult<UserChecklistResponse> {
        let checklist = self.checklist_repo
            .find_by_id(checklist_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Checklist {} not found", checklist_id)))?;

        let progress = checklist.calculate_progress();

        Ok(UserChecklistResponse {
            checklist,
            progress,
        })
    }

    async fn get_user_checklists(&self, user_id: Uuid) -> AppResult<Vec<UserChecklistResponse>> {
        let checklists = self.checklist_repo.find_by_user(user_id).await?;

        let responses = checklists
            .into_iter()
            .map(|checklist| {
                let progress = checklist.calculate_progress();
                UserChecklistResponse {
                    checklist,
                    progress,
                }
            })
            .collect();

        Ok(responses)
    }

    async fn update_step(&self, checklist_id: Uuid, dto: UpdateStepDto) -> AppResult<UserChecklistResponse> {
        let checklist = self.checklist_repo
            .update_step_status(checklist_id, dto.step_index, dto.completed)
            .await?;

        let progress = checklist.calculate_progress();

        Ok(UserChecklistResponse {
            checklist,
            progress,
        })
    }
}

