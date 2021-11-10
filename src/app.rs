use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, TimeZone};
use plist::Value;
use serde::Serialize;
use zip::ZipArchive;

#[derive(Clone, Serialize, Debug)]
pub struct App {
    id: String,
    #[serde(skip)]
    bundle_id: String,
    name: String,
    version: String,
    build: String,
    date: DateTime<FixedOffset>,
    size: u64,
    icon: Option<String>,
}

impl App {
    pub fn new(path: PathBuf, timezone: i32) -> Option<Self> {
        let file = File::open(&path).ok()?;
        let size = file.metadata().ok()?.len();
        let mut archive = ZipArchive::new(file).ok()?;
        let app_dir_path = find_app_dir(&mut archive)?;
        let (binary, bundle_id, version, build, name, icon_name) =
            find_info_plist(&mut archive, &app_dir_path)?;
        let date = binary_last_change(&mut archive, &app_dir_path, &binary, timezone)?;
        let icon = match icon_name {
            Some(name) => Some(extract_icon_base64(&mut archive, &app_dir_path, &name)?),
            None => None,
        };

        let id = path.file_stem()?.to_str()?.to_owned();
        Some(
            App {
                id,
                size,
                bundle_id,
                name,
                version,
                build,
                date,
                icon,
            },
        )
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn date(&self) -> &DateTime<FixedOffset> {
        &self.date
    }

    pub fn manifest(&self, host: &str) -> String {
        include_str!("assets/manifest.plist")
            .replacen("$HOST", host, 1)
            .replacen("$ID", &self.id, 1)
            .replacen("$BUNDLE_IDENTIFIER", &self.bundle_id, 1)
            .replacen("$BUNDLE_VERSION", &self.version, 1)
            .replacen("$TITLE", &self.name, 1)
    }
}

fn find_app_dir(archive: &mut ZipArchive<File>) -> Option<String> {
    let app_dir = archive.by_index(1).ok()?;
    if app_dir.is_dir() && app_dir.enclosed_name()?.extension()? == "app" {
        Some(app_dir.name().to_owned())
    } else {
        None
    }
}

fn find_info_plist(
    archive: &mut ZipArchive<File>,
    app_dir_path: &str,
) -> Option<(String, String, String, String, String, Option<String>)> {
    let mut plist_zip_file = archive
        .by_name(&format!("{}{}", app_dir_path, "Info.plist"))
        .ok()?;
    let mut plist = Vec::with_capacity(plist_zip_file.size() as usize);
    plist_zip_file.read_to_end(&mut plist).ok()?;

    let plist = Value::from_reader(Cursor::new(plist)).ok()?;
    let dict = plist.as_dictionary()?;

    Some((
        dict.get("CFBundleExecutable")?.as_string()?.to_owned(),
        dict.get("CFBundleIdentifier")?.as_string()?.to_owned(),
        dict.get("CFBundleShortVersionString")?
            .as_string()?
            .to_owned(),
        dict.get("CFBundleVersion")?.as_string()?.to_owned(),
        dict.get("CFBundleDisplayName")
            .or_else(|| dict.get("CFBundleName"))?
            .as_string()?
            .to_owned(),
        dict.get("CFBundleIcons")
            .and_then(|icons| icons.as_dictionary())
            .and_then(|icons| icons.get("CFBundlePrimaryIcon"))
            .and_then(|icons| icons.as_dictionary())
            .and_then(|icons| icons.get("CFBundleIconFiles"))
            .and_then(|icons| icons.as_array())
            .and_then(|icons| icons.last())
            .and_then(|icon| icon.as_string())
            .map(|icon| icon.to_owned()),
    ))
}

fn binary_last_change(
    archive: &mut ZipArchive<File>,
    app_dir_path: &str,
    binary: &str,
    timezone: i32,
) -> Option<DateTime<FixedOffset>> {
    let binary = archive
        .by_name(&format!("{}{}", app_dir_path, binary))
        .ok()?;
    let modified = binary.last_modified();
    Some(
        FixedOffset::east(60 * 60 * timezone).ymd(
            modified.year() as i32,
            modified.month() as u32,
            modified.day() as u32,
        )
            .and_hms(
                modified.hour() as u32,
                modified.minute() as u32,
                modified.second() as u32,
            ),
    )
}

fn extract_icon_base64(
    archive: &mut ZipArchive<File>,
    app_dir_path: &str,
    icon: &str,
) -> Option<String> {
    for res in ["@3x", "@2x", ""] {
        if let Ok(mut icon_zip_file) =
        archive.by_name(&format!("{}{}{}.png", app_dir_path, icon, res))
        {
            let mut icon = Vec::with_capacity(icon_zip_file.size() as usize);
            icon_zip_file.read_to_end(&mut icon).ok()?;
            return Some(base64::encode(&icon));
        }
    }
    None
}
