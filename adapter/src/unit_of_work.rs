use crate::{database::ConnectionPool, redis::RedisClient};
use async_trait::async_trait;
use kernel::unit_of_work::UnitOfWork;
use shared::error::{AppError, AppResult};
use sqlx::{Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;

macro_rules! impl_uow_scope {
    ($trait_name:ident, $work_trait:ident) => {
        #[async_trait::async_trait]
        impl $trait_name for crate::unit_of_work::UnitOfWorkScopeImpl {
            async fn begin(&self) -> shared::error::AppResult<Box<dyn $work_trait + '_>> {
                let tx = self
                    .db
                    .as_inner()
                    .begin()
                    .await
                    .map_err(shared::error::AppError::TransactionError)?;
                Ok(Box::new(crate::unit_of_work::UnitOfWorkImpl::new(
                    tx,
                    self.kv.clone(),
                    self.ttl,
                )))
            }

            async fn begin_serializable(
                &self,
            ) -> shared::error::AppResult<Box<dyn $work_trait + '_>> {
                let mut tx = self
                    .db
                    .as_inner()
                    .begin()
                    .await
                    .map_err(shared::error::AppError::TransactionError)?;
                sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
                    .execute(&mut *tx)
                    .await
                    .map_err(shared::error::AppError::SpecificOperationError)?;
                Ok(Box::new(crate::unit_of_work::UnitOfWorkImpl::new(
                    tx,
                    self.kv.clone(),
                    self.ttl,
                )))
            }
        }
    };
}

pub mod auth;
pub mod book;
pub mod checkout;
pub mod health;
pub mod user;

pub struct UnitOfWorkScopeImpl {
    db: Arc<ConnectionPool>,
    kv: Arc<RedisClient>,
    ttl: u64,
}

impl UnitOfWorkScopeImpl {
    pub fn new(db: Arc<ConnectionPool>, kv: Arc<RedisClient>, ttl: u64) -> Self {
        Self { db, kv, ttl }
    }
}

pub struct UnitOfWorkImpl<'a> {
    tx: Mutex<Transaction<'a, Postgres>>,
    kv: Arc<RedisClient>,
    ttl: u64,
}

impl<'a> UnitOfWorkImpl<'a> {
    pub fn new(tx: Transaction<'a, Postgres>, kv: Arc<RedisClient>, ttl: u64) -> Self {
        Self {
            tx: Mutex::new(tx),
            kv,
            ttl,
        }
    }
}

#[async_trait]
impl<'a> UnitOfWork for UnitOfWorkImpl<'a> {
    async fn commit(self: Box<Self>) -> AppResult<()> {
        self.tx
            .into_inner()
            .commit()
            .await
            .map_err(AppError::TransactionError)
    }

    async fn rollback(self: Box<Self>) -> AppResult<()> {
        self.tx
            .into_inner()
            .rollback()
            .await
            .map_err(AppError::TransactionError)
    }
}
