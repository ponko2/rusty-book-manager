use crate::database::ConnectionSource;
use async_trait::async_trait;
use kernel::repository::health::HealthCheckRepository;

pub struct HealthCheckRepositoryImpl<'t, 'm> {
    source: ConnectionSource<'t, 'm>,
}

impl<'t, 'm> HealthCheckRepositoryImpl<'t, 'm> {
    pub fn new(source: impl Into<ConnectionSource<'t, 'm>>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

#[async_trait]
impl<'t, 'm> HealthCheckRepository for HealthCheckRepositoryImpl<'t, 'm> {
    async fn check_db(&self) -> bool {
        let Ok(mut conn) = self.source.acquire().await else {
            return false;
        };
        sqlx::query("SELECT 1").execute(&mut *conn).await.is_ok()
    }
}
