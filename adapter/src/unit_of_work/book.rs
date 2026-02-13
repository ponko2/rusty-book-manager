use crate::{repository::book::BookRepositoryImpl, unit_of_work::UnitOfWorkImpl};
use async_trait::async_trait;
use kernel::{
    repository::book::BookRepository,
    unit_of_work::book::{BookUnitOfWork, BookUnitOfWorkScope},
};

#[async_trait]
impl<'a> BookUnitOfWork for UnitOfWorkImpl<'a> {
    fn book_repository(&self) -> Box<dyn BookRepository + '_> {
        Box::new(BookRepositoryImpl::new(&self.tx))
    }
}

impl_uow_scope!(BookUnitOfWorkScope, BookUnitOfWork);
