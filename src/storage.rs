use std::collections::HashMap;
use std::path::PathBuf;

use tokio::task::spawn_blocking;

use crate::App;
use crate::Error;

pub struct AppsStorage(pub PathBuf);

impl AppsStorage {
    pub async fn find_apps(&self) -> Result<HashMap<String, App>, Error> {
        let mut apps = HashMap::new();
        let mut dir = tokio::fs::read_dir(&self.0).await.map_err(|_| Error::AppsStorage)?;
        while let Ok(Some(file)) = dir.next_entry().await {
            if file.file_name().to_str().map(|p| p.ends_with(".ipa")) == Some(true) {
                if let Some((id, app)) = spawn_blocking(move || {
                    App::new(file.path())
                }).await.map_err(|_| Error::AppsStorage)? {
                    apps.insert(id, app);
                }
            }
        }
        Ok(apps)
    }
}