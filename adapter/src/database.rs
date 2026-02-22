use shared::{
    config::DatabaseConfig,
    error::{AppError, AppResult},
};
use sqlx::{
    PgConnection, PgPool, Postgres, Transaction, pool::PoolConnection, postgres::PgConnectOptions,
};
use std::ops::{Deref, DerefMut};
use tokio::sync::{Mutex, MutexGuard};

pub mod model;

fn make_pg_connect_options(cfg: &DatabaseConfig) -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .username(&cfg.username)
        .password(&cfg.password)
        .database(&cfg.database)
}

#[derive(Clone)]
pub struct ConnectionPool(PgPool);

impl ConnectionPool {
    pub fn as_inner(&self) -> &PgPool {
        &self.0
    }

    pub fn into_inner(self) -> PgPool {
        self.0
    }
}

impl From<PgPool> for ConnectionPool {
    fn from(pool: PgPool) -> Self {
        Self(pool)
    }
}

pub fn connect_database_with(cfg: &DatabaseConfig) -> ConnectionPool {
    PgPool::connect_lazy_with(make_pg_connect_options(cfg)).into()
}

pub enum ConnectionGuard<'g, 't: 'g> {
    Pool(Box<PoolConnection<Postgres>>),
    Transaction(MutexGuard<'g, Transaction<'t, Postgres>>),
}

impl<'g, 't: 'g> Deref for ConnectionGuard<'g, 't> {
    type Target = PgConnection;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Pool(c) => c.as_ref(),
            Self::Transaction(tx) => tx.deref(),
        }
    }
}

impl<'g, 't: 'g> DerefMut for ConnectionGuard<'g, 't> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Pool(c) => c.as_mut(),
            Self::Transaction(tx) => tx.deref_mut(),
        }
    }
}

#[derive(Clone)]
pub enum ConnectionSource<'t, 'm> {
    Pool(PgPool),
    Transaction(&'m Mutex<Transaction<'t, Postgres>>),
}

impl<'t, 'm> ConnectionSource<'t, 'm> {
    pub fn new(source: impl Into<ConnectionSource<'t, 'm>>) -> Self {
        source.into()
    }

    pub async fn acquire(&self) -> AppResult<ConnectionGuard<'_, 't>> {
        match self {
            Self::Pool(pool) => Ok(ConnectionGuard::Pool(Box::new(
                pool.acquire()
                    .await
                    .map_err(AppError::SpecificOperationError)?,
            ))),
            Self::Transaction(mutex) => Ok(ConnectionGuard::Transaction(mutex.lock().await)),
        }
    }
}

impl<'t, 'm> From<ConnectionPool> for ConnectionSource<'t, 'm> {
    fn from(pool: ConnectionPool) -> Self {
        Self::Pool(pool.into_inner())
    }
}

impl<'t, 'm> From<PgPool> for ConnectionSource<'t, 'm> {
    fn from(pool: PgPool) -> Self {
        Self::Pool(pool)
    }
}

impl<'t, 'm> From<&'m Mutex<Transaction<'t, Postgres>>> for ConnectionSource<'t, 'm> {
    fn from(tx: &'m Mutex<Transaction<'t, Postgres>>) -> Self {
        Self::Transaction(tx)
    }
}
