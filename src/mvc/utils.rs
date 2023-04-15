use std::{sync::Arc, error::Error, fmt::Display};

use axum::{Extension};
use hyper::StatusCode;
use minijinja::context;
use minijinja_autoreload::{AutoReloader, EnvironmentGuard};
use serde::Serialize;

#[cfg(debug_assertions)]
const ERROR_500_DEBUG_START: &str = r#"
<!doctype html>
<html>
    <head>
        <title>Concert With Friends - 500 Error</title>
    </head>
<body>
<h1>Status Code 500: Internal Server Error</h1>
<div>
<para>
"#;

const ERROR_500_DEBUG_END: &str = r#"
</para>
</div>
</body>
</html>
"#;

#[cfg(not(debug_assertions))]
const ERROR_500: &str = r#"
<!doctype html>
<html>
    <head>
        <title>Concert With Friends - 500 Error</title>
    </head>
<body>
<h1>Status Code 500: Internal Server Error</h1>
</body>
</html>
"#;

const ERROR_500_PROD: &str = r#"
<!doctype html>
<html>
    <head>
        <title>Concert With Friends - 500 Error</title>
    </head>
<body>
<h1>Status Code 500: Internal Server Error</h1>
</body>
</html>
"#;

/// Gets an HTML string for a 500 error status.
/// 
/// # Arguments
/// 
/// * response - If None, returns a simple error display. If Some is Ok, that is the
///              string that gets rendered. Otherwise, returns an error display that
///              includes the error message in debug mode.
/// 
/// __Returns__: A 500 status code and a string that contains the HTML to display.
pub fn internal_error<T, E>(response: Option<Result<T, E>>) -> (StatusCode, String)
    where T: Display, E: Error
{
    match response {
        Some(value) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            value
                .and_then(|display| Ok(display.to_string()))
                .unwrap_or_else(|err| {
                    if cfg!(debug_assertions) {
                        let mut response = ERROR_500_DEBUG_START.to_string();
                        response.push_str(err.to_string().as_str());
                        response.push_str(ERROR_500_DEBUG_END);
                        response
                    } else {
                        ERROR_500_PROD.to_string()
                    }
                })
        ),
        None => (StatusCode::INTERNAL_SERVER_ERROR, ERROR_500_PROD.to_string())
    }
}

/// Renders the template with the given name.
/// 
/// If the render fails for whatever reason (file not found, render failure, etc),
/// returns an error 500 page.
/// 
/// # Arguments
/// 
/// * `ext` - The template engine used to get the template
/// * `template` - The name of the template to retrieve
/// 
/// Returns a status code (200 on success, 500 on failure) and the rendered template.
/// 
/// __See Also__
/// * render_template_ctx
pub fn render_template(auto_reloader: Arc<AutoReloader>, template: &str) -> (StatusCode, String) {
    return render_template_ctx(auto_reloader, template, "");
    
}

/// Renders the template with the given name, using the provided context.
/// 
/// If the render fails for whatever reason (file not found, render failure, etc),
/// returns an error 500 page.
/// 
/// # Arguments
/// 
/// * `ext` - The template engine used to get the template
/// * `template` - The name of the template to retrieve
/// * `context` - The context used when rendering the template. See `minijinja::context!`
/// 
/// __Returns__ a status code (200 on success, 500 on failure) and the rendered template.
/// 
/// __See Also__
/// * render_template_ctx
pub fn render_template_ctx<S>(
    auto_reloader: Arc<AutoReloader>,
    template: &str,
    context: S) -> (StatusCode, String)
        where S: Serialize
{
    let result = auto_reloader
        .acquire_env()
        .or_else(|err| {
            tracing::error!("Failed to get template environment");
            Err(internal_error::<String, minijinja::Error>(None))
        })
        .and_then(|env| {
            env
                .get_template(template)
                .or_else(|err| {
                    tracing::error!("Failed to get template {}", template);
                    Err(get_error_500(&err, &env))
                })
        })
        .and_then(|template| {
            template
                .render(context)
                .or_else(|err| Err(internal_error::<String, minijinja::Error>(None)))
        });
    match result {
        Ok(value) => (StatusCode::OK, value),
        Err(err) => err
    }
}

fn get_error_500<E>(error: &E, env: &EnvironmentGuard) -> (StatusCode, String)
    where E: Error
{
    let error_500 = env
    .get_template("internal_error.j2");

    let response = match error_500 {
        Ok(template) => {
            let template_render2 = template.render(context!(
                debug => cfg!(debug_assertions),
                error_message => error.to_string()));
            
            internal_error(Some(template_render2))
        },
        Err(err) => {
            tracing::error!("Failed to get template internal_error.j2");
            internal_error::<String, minijinja::Error>(Some(Err(err)))
        }
    };

    response
}