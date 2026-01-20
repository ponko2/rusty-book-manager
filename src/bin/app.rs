use adapter::{database::connect_database_with, redis::RedisClient};
use anyhow::{Context, Result};
use api::route::{auth, v1};
use axum::{Router, http::Method};
use opentelemetry::global;
use registry::AppRegistryImpl;
use shared::{
    config::AppConfig,
    env::{Environment, which},
};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::net::TcpListener;
use tower_http::{
    LatencyUnit,
    cors::{self, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(debug_assertions)]
use api::openapi::ApiDoc;
#[cfg(debug_assertions)]
use utoipa::OpenApi;
#[cfg(debug_assertions)]
use utoipa_redoc::{Redoc, Servable};

fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_headers(cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(cors::Any)
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    bootstrap().await
}

fn init_logger() -> Result<()> {
    let log_level = match which() {
        Environment::Development => "debug",
        Environment::Production => "info",
    };

    let host = std::env::var("JAEGER_HOST")?;
    let port = std::env::var("JAEGER_PORT")?;
    let endpoint = format!("{host}:{port}");

    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_endpoint(endpoint)
        .with_service_name("book-manager")
        .with_auto_split_batch(true)
        .with_max_packet_size(8192)
        .install_simple()?;

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());
    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .json();

    tracing_subscriber::registry()
        .with(subscriber)
        .with(env_filter)
        .with(opentelemetry)
        .try_init()?;

    Ok(())
}

async fn bootstrap() -> Result<()> {
    let app_config = AppConfig::new()?;
    let pool = connect_database_with(&app_config.database);
    let kv = Arc::new(RedisClient::new(&app_config.redis)?);

    let registry = Arc::new(AppRegistryImpl::new(pool, kv, app_config));

    let router = Router::new().merge(v1::routes()).merge(auth::routes());

    #[cfg(debug_assertions)]
    let router = router.merge(Redoc::with_url("/docs", ApiDoc::openapi()));

    let app = router
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(cors())
        .with_state(registry);

    let addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 8080);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Unexpected error happened in server")
        .inspect_err(|e| {
            tracing::error!(
                error.cause_chain = ?e,error.message = %e, "Unexpected error"
            )
        })
}

async fn shutdown_signal() {
    fn purge_spans() {
        global::shutdown_tracer_provider();
    }

    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await
            .expect("Failed to receive SIGTERM signal");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Ctrl-Cを受信しました。");
            purge_spans()
        },
        _ = terminate => {
            tracing::info!("SIGTERMを受信しました。");
            purge_spans()
        }
    }
}
