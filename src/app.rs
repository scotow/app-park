use std::path::{PathBuf, Path};
use std::fs::File;
use zip::ZipArchive;
use std::io::{Read, Cursor};
use plist::Value;
use serde::Serialize;
use chrono::{Utc, TimeZone, DateTime};

#[derive(Clone, Serialize, Debug)]
pub struct App {
    id: String,
    bundle_id: String,
    name: String,
    version: String,
    build: String,
    date: DateTime<Utc>,
    icon: Option<String>,
}

impl App {
    pub fn new(path: PathBuf) -> (String, Self) {
        let mut plist = None;
        let mut date = None;
        let mut icon: Option<Vec<u8>> = None;

        let file = File::open(&path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        for i in 0..archive.len() {

            let mut zip_file = archive.by_index(i).unwrap();
            let path = Path::new(zip_file.name());
            let components = path.components().map(|c| c.as_os_str().to_str().unwrap()).collect::<Vec<_>>();
            if components.len() != 3 || components[0] != "Payload" || !components[1].ends_with(".app") {
                continue;
            }

            if components[2] == "Info.plist" {
                let mut data = Vec::with_capacity(zip_file.size() as usize);
                zip_file.read_to_end(&mut data).unwrap();
                plist = Some(data);
            } else if components[2] == components[1].trim_end_matches(".app") {
                let modified = zip_file.last_modified();
                date = Some(Utc.ymd(modified.year() as i32, modified.month() as u32, modified.day() as u32)
                    .and_hms(modified.hour() as u32, modified.minute() as u32, modified.second() as u32));
            } else if components[2].starts_with("AppIcon") && components[2].ends_with(".png") {
                if icon.is_none() || (zip_file.size() < 32_000 && icon.as_ref().map(|i| zip_file.size() > i.len() as u64) == Some(true)) {
                    let mut data = Vec::with_capacity(zip_file.size() as usize);
                    zip_file.read_to_end(&mut data).unwrap();
                    icon = Some(data);
                }
            }
        }

        match (plist, date) {
            (Some(plist), Some(date)) => {
                let plist = Value::from_reader(Cursor::new(plist)).unwrap();
                let dict = plist.as_dictionary().unwrap();

                let bundle = dict.get("CFBundleIdentifier").unwrap().as_string().unwrap();
                let version = dict.get("CFBundleShortVersionString").unwrap().as_string().unwrap();
                let build = dict.get("CFBundleVersion").unwrap().as_string().unwrap();
                let name = dict.get("CFBundleDisplayName")
                    .or_else(|| dict.get("CFBundleName"))
                    .unwrap()
                    .as_string()
                    .unwrap();

                let id = path.file_stem().unwrap().to_str().unwrap().to_owned();
                (id.clone(), App {
                    id,
                    bundle_id: bundle.to_owned(),
                    name: name.to_owned(),
                    version: version.to_owned(),
                    build: build.to_owned(),
                    date,
                    icon: icon.map(|i| base64::encode(&i)),
                })
            },
            _ => panic!(""),
        }
    }

    pub fn manifest(&self) -> String {
        include_str!("assets/manifest.plist")
            .replace("$BUNDLE_IDENTIFIER", &self.bundle_id)
            .replace("$BUNDLE_VERSION", &self.version)
            .replace("$TITLE", &self.name)
    }
}