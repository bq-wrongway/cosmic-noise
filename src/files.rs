use std::{ffi::OsStr, path::Path};

use kira::sound::PlaybackState;

pub struct NoiseTrack {
    pub name: String,
    pub path: String,
    pub volume_level: f32,
    pub is_playing: bool,
    pub state: PlaybackState,
}
pub fn get_stem(name: &Path) -> String {
    Path::file_stem(name).unwrap().to_str().unwrap().to_string()
}
pub fn load_data() -> Vec<NoiseTrack> {
    let mut files = vec![];
    if !files.is_empty() {
        files.clear();
    }
    for entry in walkdir::WalkDir::new("/usr/bin/assets/sounds") {
        let entry = entry.unwrap();
        if entry.path().is_file() && entry.path().has_extension(&["mp3", "ogg", "flac", "wav"]) {
            files.push(NoiseTrack {
                name: get_stem(entry.path()),
                path: entry.path().to_str().unwrap().to_string(),
                volume_level: 2.,
                is_playing: false,
                state: PlaybackState::Stopped,
            });
        }
    }
    files
}
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
