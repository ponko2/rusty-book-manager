use crate::unit_of_work::health::HealthCheckUnitOfWorkScope;
use async_trait::async_trait;
use std::sync::Arc;

#[mockall::automock]
#[async_trait]
pub trait HealthCheckUseCase: Send + Sync {
    async fn check_db(&self) -> bool;
}

pub struct HealthCheckUseCaseImpl {
    scope: Arc<dyn HealthCheckUnitOfWorkScope>,
}

impl HealthCheckUseCaseImpl {
    pub fn new(scope: Arc<dyn HealthCheckUnitOfWorkScope>) -> Self {
        Self { scope }
    }
}

#[async_trait]
impl HealthCheckUseCase for HealthCheckUseCaseImpl {
    async fn check_db(&self) -> bool {
        let Ok(uow) = self.scope.begin().await else {
            return false;
        };
        uow.health_check_repository().check_db().await
    }
}
