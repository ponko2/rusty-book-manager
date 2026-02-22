use crate::{repository::health::HealthCheckRepositoryImpl, unit_of_work::UnitOfWorkImpl};
use async_trait::async_trait;
use kernel::{
    repository::health::HealthCheckRepository,
    unit_of_work::health::{HealthCheckUnitOfWork, HealthCheckUnitOfWorkScope},
};

#[async_trait]
impl<'a> HealthCheckUnitOfWork for UnitOfWorkImpl<'a> {
    fn health_check_repository(&self) -> Box<dyn HealthCheckRepository + '_> {
        Box::new(HealthCheckRepositoryImpl::new(&self.tx))
    }
}

impl_uow_scope!(HealthCheckUnitOfWorkScope, HealthCheckUnitOfWork);
