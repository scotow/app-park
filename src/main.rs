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
            let file = File::open(file.path()).unwrap();
            let mut archive = ZipArchive::new(file).unwrap();
            for i in 0..archive.len() {
                let mut zip_file = archive.by_index(i).unwrap();
                let path = Path::new(zip_file.name());
                let components = path.components().map(|c| c.as_os_str()).collect::<Vec<_>>();
                if components.len() == 3 &&
                    components[0] == "Payload" &&
                    components[1].to_str().unwrap().ends_with(".app") &&
                    components[2] == "Info.plist" {
                    let mut data = Vec::with_capacity(zip_file.size() as usize);
                    zip_file.read_to_end(&mut data).unwrap();
                    let data = Cursor::new(data);
                    let plist = Value::from_reader(data).unwrap();
                    let dict = plist.as_dictionary().unwrap();

                    let version = dict.get("CFBundleShortVersionString").unwrap().as_string().unwrap();
                    let name = dict.get("CFBundleDisplayName")
                        .or_else(|| dict.get("CFBundleName"))
                        .unwrap()
                        .as_string()
                        .unwrap();
                    dbg!(version, name);
                }
            }
        }
    }
}