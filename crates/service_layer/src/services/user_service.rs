use async_trait::async_trait;
use common::{AppResult, AppError};
use models::{UserProfile, RegisterDto, LoginDto, UpdateProfileDto, AuthResponse};
use db::UserRepository;
use auth::{JwtService, PasswordService};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[async_trait]
pub trait UserService: Send + Sync {
    async fn register(&self, dto: RegisterDto) -> AppResult<AuthResponse>;
    async fn login(&self, dto: LoginDto) -> AppResult<AuthResponse>;
    async fn get_user(&self, id: Uuid) -> AppResult<UserProfile>;
    async fn update_profile(&self, user_id: Uuid, dto: UpdateProfileDto) -> AppResult<UserProfile>;
}

pub struct UserServiceImpl {
    user_repo: Arc<dyn UserRepository>,
    jwt_service: Arc<dyn JwtService>,
    password_service: Arc<dyn PasswordService>,
}

impl UserServiceImpl {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        jwt_service: Arc<dyn JwtService>,
        password_service: Arc<dyn PasswordService>,
    ) -> Self {
        Self {
            user_repo,
            jwt_service,
            password_service,
        }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn register(&self, dto: RegisterDto) -> AppResult<AuthResponse> {
        // Validate input
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Check if user already exists
        if let Some(phone) = &dto.phone {
            if self.user_repo.find_by_phone(phone).await?.is_some() {
                return Err(AppError::ValidationError("Phone already registered".to_string()));
            }
        }

        if let Some(email) = &dto.email {
            if self.user_repo.find_by_email(email).await?.is_some() {
                return Err(AppError::ValidationError("Email already registered".to_string()));
            }
        }

        // Hash password
        let password_hash = self.password_service.hash_password(&dto.password)?;

        // Create user
        let user = self.user_repo.create(dto, password_hash).await?;

        // Generate JWT token
        let token = self.jwt_service.generate_token(user.id)?;

        Ok(AuthResponse {
            user: user.into(),
            token,
        })
    }

    async fn login(&self, dto: LoginDto) -> AppResult<AuthResponse> {
        // Validate input
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Find user
        let user = if let Some(phone) = &dto.phone {
            self.user_repo.find_by_phone(phone).await?
        } else if let Some(email) = &dto.email {
            self.user_repo.find_by_email(email).await?
        } else {
            return Err(AppError::ValidationError("Phone or email required".to_string()));
        };

        let user = user.ok_or_else(|| AppError::AuthError("Invalid credentials".to_string()))?;

        // Verify password
        let is_valid = self.password_service.verify_password(&dto.password, &user.password_hash)?;
        
        if !is_valid {
            return Err(AppError::AuthError("Invalid credentials".to_string()));
        }

        // Generate JWT token
        let token = self.jwt_service.generate_token(user.id)?;

        Ok(AuthResponse {
            user: user.into(),
            token,
        })
    }

    async fn get_user(&self, id: Uuid) -> AppResult<UserProfile> {
        let user = self.user_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

        Ok(user.into())
    }

    async fn update_profile(&self, user_id: Uuid, dto: UpdateProfileDto) -> AppResult<UserProfile> {
        // Validate input
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        let user = self.user_repo.update_profile(user_id, dto).await?;

        Ok(user.into())
    }
}

