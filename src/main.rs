mod mvc;
mod settings;

use axum::{error_handling::HandleErrorLayer, routing::get, BoxError, Extension, Router};
use minijinja::{Environment, Source};
use minijinja_autoreload::AutoReloader;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_governor::{errors::display_error, governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{event, Level};

use mvc::controllers::{
    basics::{index, read_env, static_file},
    errors::page_not_found,
};
use settings::Settings;

#[tokio::main]
async fn main() {
    // Init logging
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    event!(Level::INFO, "Starting concert with friends server");

    // Create rate limiting service
    let governor_conf = Box::new(GovernorConfigBuilder::default().finish().unwrap());

    // Load global app settings
    let settings = Arc::new(Settings::new().unwrap());
    let autoreload_templates = settings.autoreload_templates;

    // Create template loader service
    let reloader = Arc::new(AutoReloader::new(move |notifier| {
        let mut env = Environment::new();
        let template_path = "static/html";

        if autoreload_templates {
            notifier.watch_path(template_path, true);
        }

        env.set_source(Source::from_path(template_path));
        Ok(env)
    }));

    // Initialize CORS layer
    let cors = CorsLayer::new().allow_origin(Any);

    // Create the app routing
    let app = Router::new()
        .route("/", get(index))
        .route("/env", get(read_env))
        .route("/static/*path", get(static_file))
        .fallback(page_not_found)
        .layer(
            ServiceBuilder::new()
                // this middleware goes above `GovernorLayer` because it will receive
                // errors returned by `GovernorLayer`
                .layer(HandleErrorLayer::new(|e: BoxError| async move {
                    display_error(e)
                }))
                .layer(GovernorLayer {
                    // We can leak this because it is created once and then
                    config: Box::leak(governor_conf),
                })
                .layer(TraceLayer::new_for_http())
                .layer(cors)
                .layer(Extension(reloader))
                .layer(Extension(settings)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
