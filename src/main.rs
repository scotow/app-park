use std::collections::HashMap;
use std::sync::Arc;

use axum::{AddExtensionLayer, handler::get, Router, Json};
use axum::extract::{Extension, Path};
use tokio::sync::Mutex;

use crate::app::App;
use axum::response::Html;
use axum::http::{HeaderMap, HeaderValue};
use axum::http::header::CONTENT_TYPE;

mod app;

type Apps = Arc<Mutex<HashMap<String, App>>>;

#[tokio::main]
async fn main() {
    let apps = Arc::new(Mutex::new(find_apps().await));

    let app = Router::new()
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/apps", get(list_apps))
        .route("/apps/:id/manifest", get(app_manifest))
        .layer(AddExtensionLayer::new(apps));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("assets/index.html"))
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

async fn find_apps() -> HashMap<String, App> {
    let mut apps = HashMap::new();
    let mut dir = tokio::fs::read_dir("apps").await.unwrap();
    while let Ok(Some(file)) = dir.next_entry().await {
        if file.file_name().to_str().map(|p| p.ends_with(".ipa")) == Some(true) {
            let (id, app) = App::new(file.path());
            apps.insert(id, app);
        }
    }
    apps
}
