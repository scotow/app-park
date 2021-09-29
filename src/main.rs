use axum::{
    handler::get,
    Router,
};
use zip::ZipArchive;
use std::fs::File;
use std::path::{Path, Component};
use std::ffi::OsStr;
use std::io::{Read, Cursor};
use plist::Value;
use crate::app::App;

mod app;

#[tokio::main]
async fn main() {
    find_apps().await;

    // // build our application with a single route
    // let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    //
    // // run it with hyper on localhost:3000
    // axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();
}

async fn find_apps() {
    let mut dir = tokio::fs::read_dir("apps").await.unwrap();
    while let Ok(Some(file)) = dir.next_entry().await {
        if file.file_name().to_str().map(|p| p.ends_with(".ipa")) == Some(true) {
            let app = App::new(file.path());
            // dbg!(app);
        }
    }
}