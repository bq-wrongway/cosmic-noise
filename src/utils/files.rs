use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::errors::{AppError, FileSystemError};
use crate::models::{NoiseTrack, SOUND_DIRECTORY, SUPPORTED_EXTENSIONS};

pub fn get_stem(name: &Path) -> String {
    log::warn!("loading path {}", name.to_string_lossy());
    name.file_stem()
        .unwrap_or_default()
        .to_os_string()
        .into_string()
        .unwrap_or_default()
}

// error handling?
pub async fn load_data() -> Result<Vec<NoiseTrack>, AppError> {
    let mut tracks = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut any_dir_exists = false;

    // Then check user data dir
    if let Some(data_path) = data_dir_exists() {
        if data_path.exists() {
            any_dir_exists = true;
            for entry in walkdir::WalkDir::new(&data_path)
                .max_depth(1)
                .follow_links(false)
                .into_iter()
            {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return Err(AppError::FileSystem(FileSystemError::DirectoryReadError)),
                };
                let path = entry.path();
                if path.is_file() {
                    if !path.has_extension(SUPPORTED_EXTENSIONS) {
                        return Err(AppError::FileSystem(FileSystemError::InvalidFileFormat));
                    }
                    let name = get_stem(path);
                    if seen.insert(name.clone()) {
                        tracks.push(NoiseTrack::new(name, path.to_path_buf()));
                    }
                }
            }
        }
    }
    // Then check user config dir
    if let Some(config_path) = config_dir_exists() {
        if config_path.exists() {
            any_dir_exists = true;
            for entry in walkdir::WalkDir::new(&config_path)
                .max_depth(1)
                .follow_links(false)
                .into_iter()
            {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return Err(AppError::FileSystem(FileSystemError::DirectoryReadError)),
                };
                let path = entry.path();
                if path.is_file() {
                    if !path.has_extension(SUPPORTED_EXTENSIONS) {
                        return Err(AppError::FileSystem(FileSystemError::InvalidFileFormat));
                    }
                    let name = get_stem(path);
                    if seen.insert(name.clone()) {
                        tracks.push(NoiseTrack::new(name, path.to_path_buf()));
                    }
                }
            }
        }
    }
    if tracks.is_empty() {
        if any_dir_exists {
            Ok(tracks)
        } else {
            Err(AppError::FileSystem(FileSystemError::DirectoryNotFound))
        }
    } else {
        Ok(tracks)
    }
}

// checks if users .config contains directory cosmic-noise/sounds
fn config_dir_exists() -> Option<PathBuf> {
    match dirs::config_local_dir() {
        Some(s) => {
            let path = s.join(SOUND_DIRECTORY);
            log::info!("Checking config dir: {}", path.display());
            match path.exists() {
                true => Some(path),
                false => None,
            }
        },
        None => None,
    }
}
// checks if users .local/share contains directory cosmic-noise/sounds
fn data_dir_exists() -> Option<PathBuf> {
    match dirs::data_local_dir() {
        Some(s) => {
            let path = s.join(SOUND_DIRECTORY);
            log::info!("Checking data dir: {}", path.display());
            match path.exists() {
                true => Some(path),
                false => None,
            }
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
