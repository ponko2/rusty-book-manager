use crate::handler::health::{health_check, health_check_db};
use axum::{Router, routing::get};
use registry::AppRegistry;

pub fn build_health_check_routers() -> Router<AppRegistry> {
    let routers = Router::new()
        .route("/", get(health_check))
        .route("/db", get(health_check_db));

    Router::new().nest("/health", routers)
}
