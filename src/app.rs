use std::path::{PathBuf, Path};
use std::fs::File;
use zip::ZipArchive;
use std::io::{Read, Cursor};
use plist::Value;

#[derive(Debug)]
pub struct App {
    id: String,
    name: String,
    version: String,
    build: String,
    image: Option<Vec<u8>>,
}

impl App {
    pub fn new(path: PathBuf) -> Self {
        let mut plist = None;
        let mut image: Option<Vec<u8>> = None;

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
            } else if components[2].starts_with("AppIcon") && components[2].ends_with(".png") {
                if image.is_none() || (zip_file.size() < 32_000 && image.as_ref().map(|i| zip_file.size() > i.len() as u64) == Some(true)) {
                    let mut data = Vec::with_capacity(zip_file.size() as usize);
                    zip_file.read_to_end(&mut data).unwrap();
                    image = Some(data);
                }
            }
        }

        if let Some(plist) = plist {
            let plist = Value::from_reader(Cursor::new(plist)).unwrap();
            let dict = plist.as_dictionary().unwrap();

            let version = dict.get("CFBundleShortVersionString").unwrap().as_string().unwrap();
            let build = dict.get("CFBundleVersion").unwrap().as_string().unwrap();
            let name = dict.get("CFBundleDisplayName")
                .or_else(|| dict.get("CFBundleName"))
                .unwrap()
                .as_string()
                .unwrap();
            App {
                id: path.file_stem().unwrap().to_str().unwrap().to_owned(),
                name: name.to_owned(),
                version: version.to_owned(),
                build: build.to_owned(),
                image,
            }
        } else {
            panic!("")
        }
    }
}