use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use kira::sound::PlaybackState;

const ALLOWED_EXT: &[&str; 4] = &["mp3", "ogg", "flac", "wav"];

pub struct NoiseTrack {
    pub name: String,
    pub path: String,
    pub volume_level: f32,
    pub state: PlaybackState,
}
//need ti handle error better
pub fn get_stem(name: &Path) -> String {
    Path::file_stem(name).unwrap().to_str().unwrap().to_string()
}
pub fn load_data() -> Vec<NoiseTrack> {
    let mut files = vec![];
    if !files.is_empty() {
        files.clear();
    }
    for entry in walkdir::WalkDir::new(get_dat_local_dir()) {
        let entry = entry.unwrap();
        if entry.path().is_file() && entry.path().has_extension(ALLOWED_EXT) {
            files.push(NoiseTrack {
                name: get_stem(entry.path()),
                path: entry.path().to_str().unwrap().to_string(),
                volume_level: 2.,
                state: PlaybackState::Stopped,
            });
        }
    }
    files
}

fn get_dat_local_dir() -> PathBuf {
    //this just loads all sound files inside of .local/share (even trash files, needs to point to the cosmic-nois)
    append_to_path(dirs::data_local_dir().unwrap(), "/cosmic-noise")
}
fn append_to_path(p: PathBuf, s: &str) -> PathBuf {
    let mut p = p.into_os_string();
    p.push(s);
    p.into()
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
