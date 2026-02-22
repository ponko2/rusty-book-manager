use crate::{
    model::{
        auth::{AccessToken, event::CreateToken},
        id::UserId,
        user::User,
    },
    unit_of_work::auth::AuthUnitOfWorkScope,
};
use async_trait::async_trait;
use shared::error::{AppError, AppResult};
use std::sync::Arc;

#[mockall::automock]
#[async_trait]
pub trait AuthUseCase: Send + Sync {
    async fn find_authorized_user(&self, access_token: &AccessToken) -> AppResult<User>;
    async fn login(&self, email: &str, password: &str) -> AppResult<(UserId, AccessToken)>;
    async fn logout(&self, access_token: AccessToken) -> AppResult<()>;
}

pub struct AuthUseCaseImpl {
    scope: Arc<dyn AuthUnitOfWorkScope>,
}

impl AuthUseCaseImpl {
    pub fn new(scope: Arc<dyn AuthUnitOfWorkScope>) -> Self {
        Self { scope }
    }
}

#[async_trait]
impl AuthUseCase for AuthUseCaseImpl {
    async fn find_authorized_user(&self, access_token: &AccessToken) -> AppResult<User> {
        let uow = self.scope.begin().await?;
        let user_id = uow
            .auth_repository()
            .fetch_user_id_from_token(access_token)
            .await?
            .ok_or(AppError::UnauthenticatedError)?;
        uow.user_repository()
            .find_current_user(user_id)
            .await?
            .ok_or(AppError::UnauthenticatedError)
    }

    async fn login(&self, email: &str, password: &str) -> AppResult<(UserId, AccessToken)> {
        let uow = self.scope.begin().await?;
        let (user_id, password_hash) = uow
            .user_repository()
            .find_password_hash_by_email(email)
            .await?;
        if !bcrypt::verify(password, &password_hash)? {
            return Err(AppError::UnauthenticatedError);
        }
        let access_token = uow
            .auth_repository()
            .create_token(CreateToken::new(user_id))
            .await?;
        uow.commit().await?;
        Ok((user_id, access_token))
    }

    async fn logout(&self, access_token: AccessToken) -> AppResult<()> {
        let uow = self.scope.begin().await?;
        uow.auth_repository().delete_token(access_token).await?;
        uow.commit().await
    }
}
