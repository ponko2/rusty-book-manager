use api::route::{auth, v1};
use axum::{Router, http::request::Builder};
use kernel::{
    model::{auth::AccessToken, id::UserId, role::Role, user::User},
    use_case::auth::MockAuthUseCase,
};
use registry::MockAppRegistryExt;
use rstest::fixture;
use std::sync::Arc;

pub fn v1(endpoint: &str) -> String {
    format!("/api/v1{}", endpoint)
}

pub fn make_router(registry: MockAppRegistryExt) -> Router {
    Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
        .with_state(Arc::new(registry))
}

#[fixture]
pub fn fixture_registry() -> MockAppRegistryExt {
    MockAppRegistryExt::new()
}

#[fixture]
pub fn fixture(mut fixture_registry: MockAppRegistryExt) -> MockAppRegistryExt {
    fixture_registry.expect_auth_use_case().returning(|| {
        let mut mock_auth_use_case = MockAuthUseCase::new();
        mock_auth_use_case
            .expect_find_authorized_user()
            .returning(|_| {
                Ok(User {
                    id: UserId::new(),
                    name: "dummy-user".to_string(),
                    email: "dummy@example.com".to_string(),
                    role: Role::User,
                })
            });
        mock_auth_use_case
            .expect_login()
            .returning(|_, _| Ok((UserId::new(), AccessToken("dummy".into()))));
        Arc::new(mock_auth_use_case)
    });
    fixture_registry
}

pub trait TestRequestExt {
    fn bearer(self) -> Builder;
}

impl TestRequestExt for Builder {
    fn bearer(self) -> Builder {
        self.header("Authorization", "Bearer dummy")
    }
}

#[macro_export]
macro_rules! deserialize_json {
    ($res:expr, $target:ty) => {{
        use tokio_stream::StreamExt;

        let mut bytes = Vec::new();
        let body = $res.into_body();
        let mut stream = body.into_data_stream();
        while let Ok(Some(chunk)) = stream.try_next().await {
            bytes.extend_from_slice(&chunk[..]);
        }
        let body: $target = serde_json::from_slice(&bytes)?;
        body
    }};
}
