use kira::sound::PlaybackState;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::app::{self, Error};

const ALLOWED_EXT: &[&str; 4] = &["mp3", "ogg", "flac", "wav"];
#[derive(Debug, Clone)]
pub struct NoiseTrack {
    pub name: String,
    pub path: PathBuf,
    pub volume_level: f32,
    pub state: PlaybackState,
}
//need ti handle error better
pub fn get_stem(name: &Path) -> String {
    name.file_stem()
        .unwrap_or(OsStr::new("###"))
        .to_os_string()
        .into_string()
        .unwrap_or(String::from("$$$"))
}

// error handling?
pub fn load_data() -> Result<Vec<NoiseTrack>, app::Error> {
    let d = get_local_dir().ok_or(Error::FileSystem)?;

    walkdir::WalkDir::new(d)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|e| e.path().has_extension(ALLOWED_EXT))
        .map(|e| {
            Ok(NoiseTrack {
                name: get_stem(e.path()),
                path: e.path().to_path_buf(),
                volume_level: 2.,
                state: PlaybackState::Stopped,
            })
        })
        .collect()
}

fn get_local_dir() -> Option<PathBuf> {
    match dirs::data_local_dir() {
        Some(pb) => Some(pb.join("cosmic-noise")),
        None => {
            log::warn!("could not access/read local directory");
            None
        }
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
