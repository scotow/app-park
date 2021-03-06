use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{AddExtensionLayer, Json, Router};
use axum::body::StreamBody;
use axum::extract::{Extension, Path};
use axum::http::{HeaderMap, HeaderValue};
use axum::http::header::{CONTENT_LENGTH, CONTENT_TYPE, HOST};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use structopt::StructOpt;
use tokio::fs::File;
use tokio::sync::{mpsc, Mutex};
use tokio_util::io::ReaderStream;

use crate::app::App;
use crate::error::Error;
use crate::options::Options;
use crate::storage::AppsStorage;

mod app;
mod error;
mod options;
mod storage;

type Apps = Arc<Mutex<HashMap<String, App>>>;

#[tokio::main]
async fn main() {
    let options = Options::from_args();

    let storage = Arc::new(AppsStorage::new(options.storage, options.timezone));
    let apps = Arc::new(Mutex::new(storage.find_apps().await.unwrap()));

    if options.watch_storage {
        let storage = Arc::clone(&storage);
        let apps = Arc::clone(&apps);
        tokio::task::spawn(async move {
            watch_storage(storage, apps).await.unwrap();
        });
    }

    let router = Router::new()
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/reload", post(reload_apps))
        .route("/apps", get(list_apps))
        .route("/apps/:id/manifest", get(app_manifest))
        .route("/apps/:id/ipa", get(app_ipa))
        .layer(AddExtensionLayer::new(storage))
        .layer(AddExtensionLayer::new(apps));

    axum::Server::bind(&SocketAddr::new(options.address, options.port))
        .serve(router.into_make_service())
        .await
        .unwrap();
}

pub async fn watch_storage(storage: Arc<AppsStorage>, apps: Apps) -> notify::Result<()> {
    let (tx, mut rx) = mpsc::channel(1);

    let mut watcher = RecommendedWatcher::new(move |res| {
        tx.blocking_send(res).unwrap();
    })?;
    watcher.watch(storage.path(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(_event) => {
                *apps.lock().await = storage.find_apps().await.unwrap();
            },
            Err(err) => eprintln!("watch error: {:?}", err),
        }
    }

    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(include_str!("assets/index.html"))
}

async fn reload_apps(
    Extension(storage): Extension<Arc<AppsStorage>>,
    Extension(apps): Extension<Apps>,
) -> Result<(), Error> {
    *apps.lock().await = storage.find_apps().await?;
    Ok(())
}

async fn list_apps(Extension(apps): Extension<Apps>) -> Json<Vec<App>> {
    let mut apps = apps.lock().await.values().cloned().collect::<Vec<_>>();
    apps.sort_by(|a, b| a.date().cmp(b.date()).reverse());
    Json(apps)
}

async fn app_manifest(
    headers: HeaderMap,
    Path(id): Path<String>,
    Extension(apps): Extension<Apps>,
) -> Result<(HeaderMap, String), Error> {
    let manifest = apps
        .lock()
        .await
        .get(&id)
        .ok_or(Error::InvalidApp)?
        .manifest(
            headers
                .get("X-Forwarded-Host")
                .or_else(|| headers.get(HOST))
                .ok_or(Error::HostHeader)?
                .to_str()
                .map_err(|_| Error::HostHeader)?,
        );
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/xml"));
    Ok((headers, manifest))
}

async fn app_ipa(
    Path(id): Path<String>,
    Extension(storage): Extension<Arc<AppsStorage>>,
) -> Result<impl IntoResponse, Error> {
    let file = File::open(storage.path().join(format!("{}.ipa", id)))
        .await
        .map_err(|_| Error::AppBinary)?;
    let size = file.metadata().await.map_err(|_| Error::AppMetadata)?.len();
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(CONTENT_LENGTH, HeaderValue::from(size));
    Ok((headers, StreamBody::new(ReaderStream::new(file))))
}
