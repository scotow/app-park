use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tokio::task::spawn_blocking;

use crate::App;
use crate::Error;

pub struct AppsStorage {
    path: PathBuf,
    timezone: i32,
}

impl AppsStorage {
    pub fn new(path: PathBuf, timezone: i32) -> Self {
        Self {
            path,
            timezone,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn find_apps(&self) -> Result<HashMap<String, App>, Error> {
        let mut apps = HashMap::new();
        let mut dir = tokio::fs::read_dir(&self.path)
            .await
            .map_err(|_| Error::AppsStorage)?;
        while let Ok(Some(file)) = dir.next_entry().await {
            if file.file_name().to_str().map(|p| p.ends_with(".ipa")) != Some(true) {
                continue;
            }
            let timezone = self.timezone;
            if let Some(app) = spawn_blocking(move || App::new(file.path(), timezone))
                .await
                .map_err(|_| Error::AppsStorage)?
            {
                apps.insert(app.id().to_owned(), app);
            }
        }
        Ok(apps)
    }
}
