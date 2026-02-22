use adapter::{database::ConnectionPool, redis::RedisClient, unit_of_work::UnitOfWorkScopeImpl};
use kernel::use_case::{
    auth::{AuthUseCase, AuthUseCaseImpl},
    book::{BookUseCase, BookUseCaseImpl},
    checkout::{CheckoutUseCase, CheckoutUseCaseImpl},
    health::{HealthCheckUseCase, HealthCheckUseCaseImpl},
    user::{UserUseCase, UserUseCaseImpl},
};
use shared::config::AppConfig;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppRegistryImpl {
    health_check_use_case: Arc<dyn HealthCheckUseCase>,
    book_use_case: Arc<dyn BookUseCase>,
    auth_use_case: Arc<dyn AuthUseCase>,
    user_use_case: Arc<dyn UserUseCase>,
    checkout_use_case: Arc<dyn CheckoutUseCase>,
}

impl AppRegistryImpl {
    pub fn new(
        pool: ConnectionPool,
        redis_client: Arc<RedisClient>,
        app_config: AppConfig,
    ) -> Self {
        let scope = Arc::new(UnitOfWorkScopeImpl::new(
            Arc::new(pool.clone()),
            redis_client.clone(),
            app_config.auth.ttl,
        ));
        let health_check_use_case = Arc::new(HealthCheckUseCaseImpl::new(scope.clone()));
        let book_use_case = Arc::new(BookUseCaseImpl::new(scope.clone()));
        let auth_use_case = Arc::new(AuthUseCaseImpl::new(scope.clone()));
        let user_use_case = Arc::new(UserUseCaseImpl::new(scope.clone()));
        let checkout_use_case = Arc::new(CheckoutUseCaseImpl::new(scope.clone()));

        Self {
            health_check_use_case,
            book_use_case,
            auth_use_case,
            user_use_case,
            checkout_use_case,
        }
    }

    pub fn health_check_use_case(&self) -> Arc<dyn HealthCheckUseCase> {
        self.health_check_use_case.clone()
    }

    pub fn book_use_case(&self) -> Arc<dyn BookUseCase> {
        self.book_use_case.clone()
    }

    pub fn auth_use_case(&self) -> Arc<dyn AuthUseCase> {
        self.auth_use_case.clone()
    }

    pub fn user_use_case(&self) -> Arc<dyn UserUseCase> {
        self.user_use_case.clone()
    }

    pub fn checkout_use_case(&self) -> Arc<dyn CheckoutUseCase> {
        self.checkout_use_case.clone()
    }
}

#[mockall::automock]
pub trait AppRegistryExt {
    fn health_check_use_case(&self) -> Arc<dyn HealthCheckUseCase>;
    fn book_use_case(&self) -> Arc<dyn BookUseCase>;
    fn auth_use_case(&self) -> Arc<dyn AuthUseCase>;
    fn checkout_use_case(&self) -> Arc<dyn CheckoutUseCase>;
    fn user_use_case(&self) -> Arc<dyn UserUseCase>;
}

impl AppRegistryExt for AppRegistryImpl {
    fn health_check_use_case(&self) -> Arc<dyn HealthCheckUseCase> {
        self.health_check_use_case.clone()
    }

    fn book_use_case(&self) -> Arc<dyn BookUseCase> {
        self.book_use_case.clone()
    }

    fn auth_use_case(&self) -> Arc<dyn AuthUseCase> {
        self.auth_use_case.clone()
    }

    fn user_use_case(&self) -> Arc<dyn UserUseCase> {
        self.user_use_case.clone()
    }

    fn checkout_use_case(&self) -> Arc<dyn CheckoutUseCase> {
        self.checkout_use_case.clone()
    }
}

pub type AppRegistry = Arc<dyn AppRegistryExt + Send + Sync + 'static>;
