use crate::{repository::health::HealthCheckRepository, unit_of_work::UnitOfWork};
use async_trait::async_trait;
use shared::error::AppResult;

#[async_trait]
pub trait HealthCheckUnitOfWork: UnitOfWork {
    fn health_check_repository(&self) -> Box<dyn HealthCheckRepository + '_>;
}

#[async_trait]
pub trait HealthCheckUnitOfWorkScope: Send + Sync {
    async fn begin(&self) -> AppResult<Box<dyn HealthCheckUnitOfWork + '_>>;
    async fn begin_serializable(&self) -> AppResult<Box<dyn HealthCheckUnitOfWork + '_>>;
}

#[cfg(test)]
mockall::mock! {
    pub HealthCheckUnitOfWork {}

    #[async_trait]
    impl UnitOfWork for HealthCheckUnitOfWork {
        async fn commit(self: Box<Self>) -> AppResult<()>;
        async fn rollback(self: Box<Self>) -> AppResult<()>;
    }

    impl HealthCheckUnitOfWork for HealthCheckUnitOfWork {
        fn health_check_repository<'a>(&'a self) -> Box<dyn HealthCheckRepository + 'a>;
    }
}

#[cfg(test)]
mockall::mock! {
    pub HealthCheckUnitOfWorkScope {}

    #[async_trait]
    impl HealthCheckUnitOfWorkScope for HealthCheckUnitOfWorkScope {
        async fn begin<'a>(&'a self) -> AppResult<Box<dyn HealthCheckUnitOfWork + 'a>>;
        async fn begin_serializable<'a>(&'a self) -> AppResult<Box<dyn HealthCheckUnitOfWork + 'a>>;
    }
}
