use crate::model::{
    id::UserId,
    user::{
        User,
        event::{CreateUser, DeleteUser, UpdateUserPassword, UpdateUserRole},
    },
};
use async_trait::async_trait;
use shared::error::AppResult;

#[mockall::automock]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, event: CreateUser) -> AppResult<User>;
    async fn delete(&self, event: DeleteUser) -> AppResult<()>;
    async fn find_all(&self) -> AppResult<Vec<User>>;
    async fn find_current_user(&self, current_user_id: UserId) -> AppResult<Option<User>>;
    async fn find_password_hash_by_email(&self, email: &str) -> AppResult<(UserId, String)>;
    async fn find_password_hash_by_user_id(&self, user_id: UserId) -> AppResult<String>;
    async fn update_password(&self, event: UpdateUserPassword) -> AppResult<()>;
    async fn update_role(&self, event: UpdateUserRole) -> AppResult<()>;
}
