use async_trait::async_trait;
use shared::error::AppResult;

pub mod auth;
pub mod book;
pub mod checkout;
pub mod health;
pub mod user;

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn commit(self: Box<Self>) -> AppResult<()>;
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}
