use crate::repository::{book::BookRepository, checkout::CheckoutRepository};
use crate::unit_of_work::UnitOfWork;
use async_trait::async_trait;
use shared::error::AppResult;

#[async_trait]
pub trait CheckoutUnitOfWork: UnitOfWork {
    fn checkout_repository(&self) -> Box<dyn CheckoutRepository + '_>;
    fn book_repository(&self) -> Box<dyn BookRepository + '_>;
}

#[async_trait]
pub trait CheckoutUnitOfWorkScope: Send + Sync {
    async fn begin(&self) -> AppResult<Box<dyn CheckoutUnitOfWork + '_>>;
    async fn begin_serializable(&self) -> AppResult<Box<dyn CheckoutUnitOfWork + '_>>;
}

#[cfg(test)]
mockall::mock! {
    pub CheckoutUnitOfWork {}

    #[async_trait]
    impl UnitOfWork for CheckoutUnitOfWork {
        async fn commit(self: Box<Self>) -> AppResult<()>;
        async fn rollback(self: Box<Self>) -> AppResult<()>;
    }

    impl CheckoutUnitOfWork for CheckoutUnitOfWork {
        fn checkout_repository<'a>(&'a self) -> Box<dyn CheckoutRepository + 'a>;
        fn book_repository<'a>(&'a self) -> Box<dyn BookRepository + 'a>;
    }
}

#[cfg(test)]
mockall::mock! {
    pub CheckoutUnitOfWorkScope {}

    #[async_trait]
    impl CheckoutUnitOfWorkScope for CheckoutUnitOfWorkScope {
        async fn begin<'a>(&'a self) -> AppResult<Box<dyn CheckoutUnitOfWork + 'a>>;
        async fn begin_serializable<'a>(&'a self) -> AppResult<Box<dyn CheckoutUnitOfWork + 'a>>;
    }
}
