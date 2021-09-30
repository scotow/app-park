use std::collections::HashMap;
use std::sync::Arc;

use axum::{AddExtensionLayer, handler::get, Json, Router};
use axum::extract::{Extension, Path};
use axum::handler::post;
use axum::http::{HeaderMap, HeaderValue};
use axum::http::header::{CONTENT_TYPE, CONTENT_LENGTH};
use axum::response::{Html, IntoResponse};
use tokio::sync::Mutex;

use crate::app::App;
use tokio::fs::File;
use axum::body::StreamBody;
use tokio_util::io::ReaderStream;

mod app;

type Apps = Arc<Mutex<HashMap<String, App>>>;

#[tokio::main]
async fn main() {
    let apps = Arc::new(Mutex::new(find_apps().await));

    let app = Router::new()
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/reload", post(reload_apps))
        .route("/apps", get(list_apps))
        .route("/apps/:id/manifest", get(app_manifest))
        .route("/apps/:id/binary", get(app_binary))
        .layer(AddExtensionLayer::new(apps));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("assets/index.html"))
}

async fn reload_apps(Extension(apps): Extension<Apps>) {
    *apps.lock().await = find_apps().await;
}

async fn list_apps(Extension(apps): Extension<Apps>) -> Json<Vec<App>> {
    let mut apps = apps.lock().await.values().cloned().collect::<Vec<_>>();
    apps.sort_by(|a, b| a.date().cmp(b.date()).reverse());
    Json(apps)
}

async fn app_manifest(Path(id): Path<String>, Extension(apps): Extension<Apps>) -> (HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
    (headers, apps.lock().await.get(&id).unwrap().manifest())
}

async fn app_binary(Path(id): Path<String>) -> impl IntoResponse {
    let file = File::open(format!("apps/{}.ipa", id)).await.unwrap();
    let size = file.metadata().await.unwrap().len();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    headers.insert(CONTENT_LENGTH, HeaderValue::from(size));
    (headers, StreamBody::new(ReaderStream::new(file)))
}

async fn find_apps() -> HashMap<String, App> {
    let mut apps = HashMap::new();
    let mut dir = tokio::fs::read_dir("apps").await.unwrap();
    while let Ok(Some(file)) = dir.next_entry().await {
        if file.file_name().to_str().map(|p| p.ends_with(".ipa")) == Some(true) {
            if let Some((id, app)) = App::new(file.path()) {
                apps.insert(id, app);
            }
        }
    }
    apps
}
