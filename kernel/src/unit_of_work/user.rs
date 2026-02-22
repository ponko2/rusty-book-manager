use crate::{
    repository::{checkout::CheckoutRepository, user::UserRepository},
    unit_of_work::UnitOfWork,
};
use async_trait::async_trait;
use shared::error::AppResult;

#[async_trait]
pub trait UserUnitOfWork: UnitOfWork {
    fn checkout_repository(&self) -> Box<dyn CheckoutRepository + '_>;
    fn user_repository(&self) -> Box<dyn UserRepository + '_>;
}

#[async_trait]
pub trait UserUnitOfWorkScope: Send + Sync {
    async fn begin(&self) -> AppResult<Box<dyn UserUnitOfWork + '_>>;
    async fn begin_serializable(&self) -> AppResult<Box<dyn UserUnitOfWork + '_>>;
}

#[cfg(test)]
mockall::mock! {
    pub UserUnitOfWork {}

    #[async_trait]
    impl UnitOfWork for UserUnitOfWork {
        async fn commit(self: Box<Self>) -> AppResult<()>;
        async fn rollback(self: Box<Self>) -> AppResult<()>;
    }

    impl UserUnitOfWork for UserUnitOfWork {
        fn checkout_repository<'a>(&'a self) -> Box<dyn CheckoutRepository + 'a>;
        fn user_repository<'a>(&'a self) -> Box<dyn UserRepository + 'a>;
    }
}

#[cfg(test)]
mockall::mock! {
    pub UserUnitOfWorkScope {}

    #[async_trait]
    impl UserUnitOfWorkScope for UserUnitOfWorkScope {
        async fn begin<'a>(&'a self) -> AppResult<Box<dyn UserUnitOfWork + 'a>>;
        async fn begin_serializable<'a>(&'a self) -> AppResult<Box<dyn UserUnitOfWork + 'a>>;
    }
}
