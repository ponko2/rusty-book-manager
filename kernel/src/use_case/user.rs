use crate::{
    model::{
        checkout::Checkout,
        user::{
            User,
            event::{CreateUser, DeleteUser, UpdateUserPassword, UpdateUserRole},
        },
    },
    unit_of_work::user::UserUnitOfWorkScope,
};
use async_trait::async_trait;
use shared::error::{AppError, AppResult};
use std::sync::Arc;

#[mockall::automock]
#[async_trait]
pub trait UserUseCase: Send + Sync {
    async fn change_password(&self, event: UpdateUserPassword) -> AppResult<()>;
    async fn change_role(&self, event: UpdateUserRole) -> AppResult<()>;
    async fn delete_user(&self, event: DeleteUser) -> AppResult<()>;
    async fn get_checkouts(&self, user_id: crate::model::id::UserId) -> AppResult<Vec<Checkout>>;
    async fn list_users(&self) -> AppResult<Vec<User>>;
    async fn register_user(&self, event: CreateUser) -> AppResult<User>;
}

pub struct UserUseCaseImpl {
    scope: Arc<dyn UserUnitOfWorkScope>,
}

impl UserUseCaseImpl {
    pub fn new(scope: Arc<dyn UserUnitOfWorkScope>) -> Self {
        Self { scope }
    }
}

#[async_trait]
impl UserUseCase for UserUseCaseImpl {
    async fn change_password(&self, event: UpdateUserPassword) -> AppResult<()> {
        let uow = self.scope.begin().await?;
        {
            let user_repository = uow.user_repository();
            let original_password_hash = user_repository
                .find_password_hash_by_user_id(event.user_id)
                .await?;
            if !bcrypt::verify(&event.current_password, &original_password_hash)? {
                return Err(AppError::UnauthenticatedError);
            }
            user_repository.update_password(event).await?;
        }
        uow.commit().await
    }

    async fn change_role(&self, event: UpdateUserRole) -> AppResult<()> {
        let uow = self.scope.begin().await?;
        uow.user_repository().update_role(event).await?;
        uow.commit().await
    }

    async fn delete_user(&self, event: DeleteUser) -> AppResult<()> {
        let uow = self.scope.begin().await?;
        uow.user_repository().delete(event).await?;
        uow.commit().await
    }

    async fn get_checkouts(&self, user_id: crate::model::id::UserId) -> AppResult<Vec<Checkout>> {
        let uow = self.scope.begin().await?;
        uow.checkout_repository()
            .find_unreturned_by_user_id(user_id)
            .await
    }

    async fn list_users(&self) -> AppResult<Vec<User>> {
        let uow = self.scope.begin().await?;
        uow.user_repository().find_all().await
    }

    async fn register_user(&self, event: CreateUser) -> AppResult<User> {
        let uow = self.scope.begin().await?;
        let user = uow.user_repository().create(event).await?;
        uow.commit().await?;
        Ok(user)
    }
}
