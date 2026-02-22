use crate::{repository::book::BookRepository, unit_of_work::UnitOfWork};
use async_trait::async_trait;
use shared::error::AppResult;

#[async_trait]
pub trait BookUnitOfWork: UnitOfWork {
    fn book_repository(&self) -> Box<dyn BookRepository + '_>;
}

#[async_trait]
pub trait BookUnitOfWorkScope: Send + Sync {
    async fn begin(&self) -> AppResult<Box<dyn BookUnitOfWork + '_>>;
    async fn begin_serializable(&self) -> AppResult<Box<dyn BookUnitOfWork + '_>>;
}

#[cfg(test)]
mockall::mock! {
    pub BookUnitOfWork {}

    #[async_trait]
    impl UnitOfWork for BookUnitOfWork {
        async fn commit(self: Box<Self>) -> AppResult<()>;
        async fn rollback(self: Box<Self>) -> AppResult<()>;
    }

    impl BookUnitOfWork for BookUnitOfWork {
        fn book_repository<'a>(&'a self) -> Box<dyn BookRepository + 'a>;
    }
}

#[cfg(test)]
mockall::mock! {
    pub BookUnitOfWorkScope {}

    #[async_trait]
    impl BookUnitOfWorkScope for BookUnitOfWorkScope {
        async fn begin<'a>(&'a self) -> AppResult<Box<dyn BookUnitOfWork + 'a>>;
        async fn begin_serializable<'a>(&'a self) -> AppResult<Box<dyn BookUnitOfWork + 'a>>;
    }
}
