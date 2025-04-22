use axum::{
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use mime_guess::from_path;
use rust_embed::RustEmbed;

const INDEX_HTML: &str = "index.html";

#[derive(RustEmbed)]
#[folder = "ui/dist"]
pub struct UiAssets;

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == INDEX_HTML {
        return index_html().await;
    }

    match UiAssets::get(path) {
        Some(content) => {
            let mime = from_path(path).first_or_octet_stream();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            ).into_response()
        }
        None => {
            if path.contains('.') {
                return not_found().await;
            }
            index_html().await
        }
    }
}

async fn index_html() -> Response {
    match UiAssets::get(INDEX_HTML) {
        Some(content) => Html(content.data).into_response(),
        None => not_found().await,
    }
}

async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
} 