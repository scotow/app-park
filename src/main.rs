use std::collections::HashMap;
use std::sync::Arc;

use axum::{AddExtensionLayer, handler::get, Router, Json};
use axum::extract::Extension;
use tokio::sync::Mutex;

use crate::app::App;

mod app;

type Apps = Arc<Mutex<HashMap<String, App>>>;

#[tokio::main]
async fn main() {
    let apps = Arc::new(Mutex::new(find_apps().await));

    let app = Router::new()
        .route("/apps", get(list_apps))
        .layer(AddExtensionLayer::new(apps));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn list_apps(state: Extension<Apps>) -> Json<Vec<App>> {
    Json(state.0.lock().await.values().cloned().collect())
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
