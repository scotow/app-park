use std::collections::HashMap;
use std::sync::Arc;

use axum::{AddExtensionLayer, handler::get, Json, Router};
use axum::extract::{Extension, Path};
use axum::handler::post;
use axum::http::{HeaderMap, HeaderValue};
use axum::http::header::{CONTENT_TYPE, CONTENT_LENGTH, HOST};
use axum::response::{Html, IntoResponse};
use tokio::sync::Mutex;

use crate::app::App;
use tokio::fs::File;
use axum::body::StreamBody;
use tokio_util::io::ReaderStream;
use crate::error::Error;

mod app;
mod error;

type Apps = Arc<Mutex<HashMap<String, App>>>;

#[tokio::main]
async fn main() {
    let apps = Arc::new(Mutex::new(find_apps().await.unwrap()));

    let app = Router::new()
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/reload", post(reload_apps))
        .route("/apps", get(list_apps))
        .route("/apps/:id/manifest", get(app_manifest))
        .route("/apps/:id/ipa", get(app_ipa))
        .layer(AddExtensionLayer::new(apps));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("assets/index.html"))
}

async fn reload_apps(Extension(apps): Extension<Apps>) -> Result<(), Error> {
    *apps.lock().await = find_apps().await?;
    Ok(())
}

async fn list_apps(Extension(apps): Extension<Apps>) -> Json<Vec<App>> {
    let mut apps = apps.lock().await.values().cloned().collect::<Vec<_>>();
    apps.sort_by(|a, b| a.date().cmp(b.date()).reverse());
    Json(apps)
}

async fn app_manifest(headers: HeaderMap, Path(id): Path<String>, Extension(apps): Extension<Apps>) -> Result<(HeaderMap, String), Error> {
    let manifest = apps.lock().await.get(&id)
        .ok_or(Error::InvalidApp)?
        .manifest(
            headers.get(HOST).ok_or(Error::HostHeader)?
                .to_str().map_err(|_| Error::HostHeader)?
        );
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
    Ok((headers, manifest))
}

async fn app_ipa(Path(id): Path<String>) -> Result<impl IntoResponse, Error> {
    let file = File::open(format!("apps/{}.ipa", id)).await.map_err(|_| Error::AppBinary)?;
    let size = file.metadata().await.map_err(|_| Error::AppMetadata)?.len();
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    headers.insert(CONTENT_LENGTH, HeaderValue::from(size));
    Ok((headers, StreamBody::new(ReaderStream::new(file))))
}

async fn find_apps() -> Result<HashMap<String, App>, Error> {
    let mut apps = HashMap::new();
    let mut dir = tokio::fs::read_dir("apps").await.map_err(|_| Error::AppsStorage)?;
    while let Ok(Some(file)) = dir.next_entry().await {
        if file.file_name().to_str().map(|p| p.ends_with(".ipa")) == Some(true) {
            if let Some((id, app)) = App::new(file.path()) {
                apps.insert(id, app);
            }
        }
    }
    Ok(apps)
}
