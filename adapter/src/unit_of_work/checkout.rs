use crate::{
    repository::{book::BookRepositoryImpl, checkout::CheckoutRepositoryImpl},
    unit_of_work::UnitOfWorkImpl,
};
use async_trait::async_trait;
use kernel::{
    repository::{book::BookRepository, checkout::CheckoutRepository},
    unit_of_work::checkout::{CheckoutUnitOfWork, CheckoutUnitOfWorkScope},
};

#[async_trait]
impl<'a> CheckoutUnitOfWork for UnitOfWorkImpl<'a> {
    fn checkout_repository(&self) -> Box<dyn CheckoutRepository + '_> {
        Box::new(CheckoutRepositoryImpl::new(&self.tx))
    }

    fn book_repository(&self) -> Box<dyn BookRepository + '_> {
        Box::new(BookRepositoryImpl::new(&self.tx))
    }
}

impl_uow_scope!(CheckoutUnitOfWorkScope, CheckoutUnitOfWork);
