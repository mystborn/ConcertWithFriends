use axum::{
    body::{self, Empty, Full},
    error_handling::HandleErrorLayer,
    extract::Path,
    response::{Html, IntoResponse, Response},
    routing::get,
    BoxError, Extension, Router,
};
use hyper::StatusCode;
use minijinja::{Environment, Source};
use minijinja_autoreload::AutoReloader;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_governor::{errors::display_error, governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{event, Level};

mod settings;
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

async fn index(Extension(template_loader): Extension<Arc<AutoReloader>>) -> Html<String> {
    let template = template_loader
        .acquire_env()
        .unwrap()
        .get_template("index.j2")
        .unwrap()
        .render("")
        .unwrap();
    Html(template)
}

async fn page_not_found(
    Extension(template_loader): Extension<Arc<AutoReloader>>,
) -> impl IntoResponse {
    let template = template_loader
        .acquire_env()
        .unwrap()
        .get_template("page_not_found.j2")
        .unwrap()
        .render("")
        .unwrap();
    (StatusCode::NOT_FOUND, Html(template))
}

async fn static_file(Path(path): Path<String>) -> impl IntoResponse {
    let path = format!("static/{}", path.trim_start_matches('/'));
    let mime_type = mime_guess::from_path(&path).first_or_text_plain();
    let file = std::fs::read_to_string(path);
    match file {
        Ok(contents) => Response::builder()
            .status(StatusCode::OK)
            .header(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(body::boxed(Full::from(contents)))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap(),
    }
}

async fn read_env(Extension(settings): Extension<Arc<Settings>>) -> String {
    settings.env.to_string()
}
