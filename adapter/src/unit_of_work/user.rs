use crate::{
    repository::{checkout::CheckoutRepositoryImpl, user::UserRepositoryImpl},
    unit_of_work::UnitOfWorkImpl,
};
use async_trait::async_trait;
use kernel::{
    repository::{checkout::CheckoutRepository, user::UserRepository},
    unit_of_work::user::{UserUnitOfWork, UserUnitOfWorkScope},
};

#[async_trait]
impl<'a> UserUnitOfWork for UnitOfWorkImpl<'a> {
    fn checkout_repository(&self) -> Box<dyn CheckoutRepository + '_> {
        Box::new(CheckoutRepositoryImpl::new(&self.tx))
    }

    fn user_repository(&self) -> Box<dyn UserRepository + '_> {
        Box::new(UserRepositoryImpl::new(&self.tx))
    }
}

impl_uow_scope!(UserUnitOfWorkScope, UserUnitOfWork);
