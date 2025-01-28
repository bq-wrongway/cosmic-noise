use kira::sound::PlaybackState;
use std::{
    ffi::OsStr,
    io::Error,
    path::{Path, PathBuf},
};

const ALLOWED_EXT: &[&str; 4] = &["mp3", "ogg", "flac", "wav"];

pub struct NoiseTrack {
    pub name: String,
    pub path: PathBuf,
    pub volume_level: f32,
    pub state: PlaybackState,
}
//need ti handle error better
pub fn get_stem(name: &Path) -> String {
    match Path::file_stem(name) {
        Some(s) => s.to_string_lossy().into_owned(),
        None => {
            log::error!("could not get stem from the given path");
            String::from("")
        }
    }
}

// error handling?
pub fn load_data() -> Result<Vec<NoiseTrack>, Error> {
    let mut files = vec![];
    if !files.is_empty() {
        files.clear();
    }
    match get_local_dir() {
        Some(d) => {
            for entry in walkdir::WalkDir::new(d) {
                // clet entry = entry?;
                match entry {
                    Ok(dir_entry) => {
                        if dir_entry.path().is_file() && dir_entry.path().has_extension(ALLOWED_EXT)
                        {
                            files.push(NoiseTrack {
                                name: get_stem(dir_entry.path()),
                                path: dir_entry.path().to_path_buf(),
                                volume_level: 2.,
                                state: PlaybackState::Stopped,
                            });
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Cant find or access $HOME.local/cosmic-noise/sounds directory  {}",
                            e
                        );
                    }
                }
            }
        }
        None => {
            log::error!("no directory found");
        }
    };
    Ok(files)
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
