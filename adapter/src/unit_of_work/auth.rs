use crate::{
    repository::{auth::AuthRepositoryImpl, user::UserRepositoryImpl},
    unit_of_work::UnitOfWorkImpl,
};
use async_trait::async_trait;
use kernel::{
    repository::{auth::AuthRepository, user::UserRepository},
    unit_of_work::auth::{AuthUnitOfWork, AuthUnitOfWorkScope},
};

#[async_trait]
impl<'a> AuthUnitOfWork for UnitOfWorkImpl<'a> {
    fn auth_repository(&self) -> Box<dyn AuthRepository + '_> {
        Box::new(AuthRepositoryImpl::new(self.kv.clone(), self.ttl))
    }

    fn user_repository(&self) -> Box<dyn UserRepository + '_> {
        Box::new(UserRepositoryImpl::new(&self.tx))
    }
}

impl_uow_scope!(AuthUnitOfWorkScope, AuthUnitOfWork);
