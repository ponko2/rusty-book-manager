use crate::{
    repository::{auth::AuthRepository, user::UserRepository},
    unit_of_work::UnitOfWork,
};
use async_trait::async_trait;
use shared::error::AppResult;

#[async_trait]
pub trait AuthUnitOfWork: UnitOfWork {
    fn auth_repository(&self) -> Box<dyn AuthRepository + '_>;
    fn user_repository(&self) -> Box<dyn UserRepository + '_>;
}

#[async_trait]
pub trait AuthUnitOfWorkScope: Send + Sync {
    async fn begin(&self) -> AppResult<Box<dyn AuthUnitOfWork + '_>>;
    async fn begin_serializable(&self) -> AppResult<Box<dyn AuthUnitOfWork + '_>>;
}

#[cfg(test)]
mockall::mock! {
    pub AuthUnitOfWork {}

    #[async_trait]
    impl UnitOfWork for AuthUnitOfWork {
        async fn commit(self: Box<Self>) -> AppResult<()>;
        async fn rollback(self: Box<Self>) -> AppResult<()>;
    }

    impl AuthUnitOfWork for AuthUnitOfWork {
        fn auth_repository<'a>(&'a self) -> Box<dyn AuthRepository + 'a>;
        fn user_repository<'a>(&'a self) -> Box<dyn UserRepository + 'a>;
    }
}

#[cfg(test)]
mockall::mock! {
    pub AuthUnitOfWorkScope {}

    #[async_trait]
    impl AuthUnitOfWorkScope for AuthUnitOfWorkScope {
        async fn begin<'a>(&'a self) -> AppResult<Box<dyn AuthUnitOfWork + 'a>>;
        async fn begin_serializable<'a>(&'a self) -> AppResult<Box<dyn AuthUnitOfWork + 'a>>;
    }
}
