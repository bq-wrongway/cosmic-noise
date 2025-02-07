use kira::sound::PlaybackState;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::app::{self, Error};

const ALLOWED_EXT: &[&str; 4] = &["mp3", "ogg", "flac", "wav"];
const SOUND_FILE: &str = "cosmic-noise/sounds/";
#[derive(Debug, Clone)]
pub struct NoiseTrack {
    pub name: String,
    pub path: PathBuf,
    pub volume_level: f32,
    pub state: PlaybackState,
}
pub fn get_stem(name: &Path) -> String {
    log::warn!("loading path {}", name.to_string_lossy());
    name.file_stem()
        .unwrap_or_default()
        .to_os_string()
        .into_string()
        .unwrap_or_default()
}

// error handling?
pub async fn load_data() -> Result<Vec<NoiseTrack>, app::Error> {
    let d = get_local_dir().ok_or(Error::FileSystem)?;
    walkdir::WalkDir::new(d)
        .max_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_map(|it| match it {
            Ok(entry) => {
                let path = entry.path();
                (path.is_file() && path.has_extension(ALLOWED_EXT)).then(|| {
                    Ok(NoiseTrack {
                        name: get_stem(path),
                        path: path.to_path_buf(),
                        volume_level: 2.,
                        state: PlaybackState::Stopped,
                    })
                })
            }
            Err(_) => Some(Err(Error::FileSystem)),
        })
        .collect()
    // .filter_map(|f| f.ok())
    // .filter(|e| e.path().has_extension(ALLOWED_EXT))
    // .map(|e| {
    // Ok(NoiseTrack {
    //     name: get_stem(e.path()),
    //     path: e.path().to_path_buf(),
    //     volume_level: 2.,
    //     state: PlaybackState::Stopped,
    // })
    // })
    // .collect()
}
//check if resource directories exist and return the path of one that does

fn get_local_dir() -> Option<PathBuf> {
    match data_dir_exists() {
        Some(s) => {
            println!("here : {:?}", s);
            Some(s)
        }
        None => config_dir_exists(),
    }
}

// checks if users .config contains directory cosmic-noise/sounds
fn config_dir_exists() -> Option<PathBuf> {
    match dirs::config_local_dir() {
        Some(s) => match s.join(SOUND_FILE).exists() {
            true => Some(s.join(SOUND_FILE)),
            false => None,
        },
        None => None,
    }
}
// checks if users .local/share contains directory cosmic-noise/sounds
fn data_dir_exists() -> Option<PathBuf> {
    match dirs::data_local_dir() {
        Some(s) => match s.join(SOUND_FILE).exists() {
            true => Some(s.join(SOUND_FILE)),
            false => None,
        },
        None => None,
    }
}

// a way to check extension and allow only from the extension allow list
pub trait FileExtension {
    fn has_extension<S: AsRef<str>>(&self, extensions: &[S]) -> bool;
}

impl<P: AsRef<Path>> FileExtension for P {
    fn has_extension<S: AsRef<str>>(&self, extensions: &[S]) -> bool {
        if let Some(extension) = self.as_ref().extension().and_then(OsStr::to_str) {
            return extensions
                .iter()
                .any(|x| x.as_ref().eq_ignore_ascii_case(extension));
        }

        false
    }
}
