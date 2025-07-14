use crate::errors::{AppError, AudioError};
use crate::models::{AudioSettings, AudioStats, NoiseTrack};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings};
use kira::sound::{FromFileError, PlaybackState};
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, Tween};
use std::collections::HashMap;
use std::path::Path;

pub struct AudioSystem {
    manager: Option<AudioManager<DefaultBackend>>,
    playing_handles: HashMap<usize, StreamingSoundHandle<FromFileError>>,
    global_state: PlaybackState,
    default_settings: AudioSettings,
}

#[derive(Debug, Clone)]
pub enum AudioCommand {
    Play(usize),
    Pause(usize),
    Resume(usize),
    Stop(usize),
    SetVolume { track_id: usize, volume: f32 },
    StopAll,
    PauseAll,
    ResumeAll,
    SetMasterVolume(f32),
}

#[derive(Debug, Clone)]
pub enum AudioEvent {
    TrackStarted(usize),
    TrackPaused(usize),
    TrackResumed(usize),
    TrackStopped(usize),
    TrackFinished(usize),
    VolumeChanged { track_id: usize, volume: f32 },
    Error(AudioError),
    MasterVolumeChanged(f32),
}

impl AudioSystem {
    /// Create a new audio system with default settings
    pub fn new() -> Result<Self, AppError> {
        Self::with_settings(AudioSettings::default())
    }

    /// Create a new audio system with custom settings
    pub fn with_settings(settings: AudioSettings) -> Result<Self, AppError> {
        let manager_settings = AudioManagerSettings {
            // Configure based on our settings
            ..AudioManagerSettings::default()
        };

        let manager = AudioManager::<DefaultBackend>::new(manager_settings)
            .map_err(|_| AppError::Audio(AudioError::InitializationFailed))?;

        Ok(Self {
            manager: Some(manager),
            playing_handles: HashMap::new(),
            global_state: PlaybackState::Stopped,
            default_settings: settings,
        })
    }

    /// Initialize the audio system (can be called multiple times safely)
    pub fn initialize(&mut self) -> Result<(), AppError> {
        if self.manager.is_none() {
            *self = Self::new()?;
        }
        Ok(())
    }

    /// Check if the audio system is initialized
    pub fn is_initialized(&self) -> bool {
        self.manager.is_some()
    }

    /// Get the current global playback state
    pub fn global_state(&self) -> PlaybackState {
        self.global_state
    }

    /// Get the number of currently playing tracks
    pub fn active_tracks_count(&self) -> usize {
        self.playing_handles.len()
    }

    /// Check if a specific track is playing
    pub fn is_track_playing(&self, track_id: usize) -> bool {
        self.playing_handles
            .get(&track_id)
            .map(|handle| matches!(handle.state(), PlaybackState::Playing))
            .unwrap_or(false)
    }

    /// Get the state of a specific track
    pub fn track_state(&self, track_id: usize) -> PlaybackState {
        self.playing_handles
            .get(&track_id)
            .map(|handle| handle.state())
            .unwrap_or(PlaybackState::Stopped)
    }

    /// Get the current master volume
    pub fn master_volume(&self) -> f32 {
        self.default_settings.master_volume
    }

    /// Set the master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.default_settings.master_volume = volume;
    }

    /// Process an audio command
    pub fn process_command(
        &mut self,
        command: AudioCommand,
        tracks: &mut [NoiseTrack],
    ) -> Result<Vec<AudioEvent>, AppError> {
        let mut events = Vec::new();

        match command {
            AudioCommand::Play(track_id) => {
                events.extend(self.play_track(track_id, tracks)?);
            }
            AudioCommand::Pause(track_id) => {
                events.extend(self.pause_track(track_id, tracks)?);
            }
            AudioCommand::Resume(track_id) => {
                events.extend(self.resume_track(track_id, tracks)?);
            }
            AudioCommand::Stop(track_id) => {
                events.extend(self.stop_track(track_id, tracks)?);
            }
            AudioCommand::SetVolume { track_id, volume } => {
                events.extend(self.set_track_volume(track_id, volume, tracks)?);
            }
            AudioCommand::StopAll => {
                events.extend(self.stop_all_tracks(tracks)?);
            }
            AudioCommand::PauseAll => {
                events.extend(self.pause_all_tracks(tracks)?);
            }
            AudioCommand::ResumeAll => {
                events.extend(self.resume_all_tracks(tracks)?);
            }
            AudioCommand::SetMasterVolume(volume) => {
                // Implement master volume control
                log::info!("Master volume set to: {volume}");

                // Update the master volume in settings
                self.default_settings.master_volume = volume;

                // Save master volume to configuration
                if let Err(e) = crate::config::ConfigManager::save_master_volume(volume) {
                    log::error!("Failed to save master volume to configuration: {e}");
                }

                // Apply master volume to all currently playing tracks
                let tween = self.create_tween();
                for (track_id, handle) in self.playing_handles.iter_mut() {
                    // Calculate effective volume: combine track volume with master volume
                    // In dB, we add the values: track_volume + master_volume
                    let effective_volume = tracks[*track_id].volume_level + volume;
                    // Clamp to valid range
                    let clamped_volume = effective_volume.clamp(-60.0, 0.0);
                    handle.set_volume(clamped_volume, tween);
                }

                events.push(AudioEvent::MasterVolumeChanged(volume));
            }
        }

        Ok(events)
    }

    /// Play a track by index
    fn play_track(
        &mut self,
        track_id: usize,
        tracks: &mut [NoiseTrack],
    ) -> Result<Vec<AudioEvent>, AppError> {
        let mut events = Vec::new();

        if track_id >= tracks.len() {
            return Err(AppError::Audio(AudioError::PlaybackError(
                "Track index out of bounds".to_string(),
            )));
        }

        // Check if track is already playing
        let tween = self.create_tween();
        if let Some(handle) = self.playing_handles.get_mut(&track_id) {
            match handle.state() {
                PlaybackState::Playing => {
                    // Pause if already playing
                    handle.pause(tween);
                    tracks[track_id].state = PlaybackState::Paused;
                    events.push(AudioEvent::TrackPaused(track_id));
                }
                PlaybackState::Paused => {
                    // Resume if paused
                    handle.resume(tween);
                    tracks[track_id].state = PlaybackState::Playing;
                    events.push(AudioEvent::TrackResumed(track_id));
                }
                _ => {
                    // Handle is in stopped state, remove it and create new one
                    self.playing_handles.remove(&track_id);
                    events.extend(self.start_new_track(track_id, tracks)?);
                }
            }
        } else {
            // Start new track
            events.extend(self.start_new_track(track_id, tracks)?);
        }

        self.update_global_state();
        Ok(events)
    }

    /// Start a new track from the beginning
    fn start_new_track(
        &mut self,
        track_id: usize,
        tracks: &mut [NoiseTrack],
    ) -> Result<Vec<AudioEvent>, AppError> {
        let mut events = Vec::new();
        let track_path = tracks[track_id].path.clone();
        let track_volume = tracks[track_id].volume_level;
        let track_name = tracks[track_id].name.clone();

        // Calculate effective volume: combine track volume with master volume
        let effective_volume = track_volume + self.default_settings.master_volume;
        // Clamp to valid range
        let clamped_volume = effective_volume.clamp(-60.0, 0.0);

        // Create streaming sound settings
        let settings = StreamingSoundSettings::new()
            .volume(clamped_volume)
            .loop_region(self.default_settings.loop_region.clone().unwrap_or(0.0..));

        // Load and play the sound
        let handle = self.load_and_play_sound(&track_path, settings)?;

        // Store the handle and update track state
        self.playing_handles.insert(track_id, handle);
        tracks[track_id].state = PlaybackState::Playing;

        events.push(AudioEvent::TrackStarted(track_id));
        log::info!("Started playing track: {track_name}");

        Ok(events)
    }

    /// Load and play a sound file
    fn load_and_play_sound(
        &mut self,
        path: &Path,
        settings: StreamingSoundSettings,
    ) -> Result<StreamingSoundHandle<FromFileError>, AppError> {
        let sound_data =
            StreamingSoundData::from_file(path).map_err(|e| AppError::Audio(e.into()))?;

        let manager = self
            .manager
            .as_mut()
            .ok_or(AppError::Audio(AudioError::InitializationFailed))?;

        let handle = manager
            .play(sound_data.with_settings(settings))
            .map_err(|e| {
                log::error!("Failed to play sound: {}", e);
                AppError::Audio(AudioError::HandleCreationFailed)
            })?;

        Ok(handle)
    }

    /// Pause a track by index
    fn pause_track(
        &mut self,
        track_id: usize,
        tracks: &mut [NoiseTrack],
    ) -> Result<Vec<AudioEvent>, AppError> {
        let mut events = Vec::new();

        let tween = self.create_tween();
        if let Some(handle) = self.playing_handles.get_mut(&track_id) {
            if matches!(handle.state(), PlaybackState::Playing) {
                handle.pause(tween);
                tracks[track_id].state = PlaybackState::Paused;
                events.push(AudioEvent::TrackPaused(track_id));
                log::info!("Paused track: {}", tracks[track_id].name);
            }
        }

        self.update_global_state();
        Ok(events)
    }

    /// Resume a track by index
    fn resume_track(
        &mut self,
        track_id: usize,
        tracks: &mut [NoiseTrack],
    ) -> Result<Vec<AudioEvent>, AppError> {
        let mut events = Vec::new();

        let tween = self.create_tween();
        if let Some(handle) = self.playing_handles.get_mut(&track_id) {
            if matches!(handle.state(), PlaybackState::Paused) {
                handle.resume(tween);
                tracks[track_id].state = PlaybackState::Playing;
                events.push(AudioEvent::TrackResumed(track_id));
                log::info!("Resumed track: {}", tracks[track_id].name);
            }
        }

        self.update_global_state();
        Ok(events)
    }

    /// Stop a track by index
    fn stop_track(
        &mut self,
        track_id: usize,
        tracks: &mut [NoiseTrack],
    ) -> Result<Vec<AudioEvent>, AppError> {
        let mut events = Vec::new();

        let tween = self.create_tween();
        if let Some(mut handle) = self.playing_handles.remove(&track_id) {
            handle.stop(tween);
            tracks[track_id].state = PlaybackState::Stopped;
            events.push(AudioEvent::TrackStopped(track_id));
            log::info!("Stopped track: {}", tracks[track_id].name);
        }

        self.update_global_state();
        Ok(events)
    }

    /// Set volume for a specific track
    fn set_track_volume(
        &mut self,
        track_id: usize,
        volume: f32,
        tracks: &mut [NoiseTrack],
    ) -> Result<Vec<AudioEvent>, AppError> {
        let mut events = Vec::new();

        let tween = self.create_tween();
        if let Some(handle) = self.playing_handles.get_mut(&track_id) {
            // Calculate effective volume: combine track volume with master volume
            let effective_volume = volume + self.default_settings.master_volume;
            // Clamp to valid range
            let clamped_volume = effective_volume.clamp(-60.0, 0.0);
            handle.set_volume(clamped_volume, tween);
            tracks[track_id].volume_level = volume;
            events.push(AudioEvent::VolumeChanged { track_id, volume });
            log::info!(
                "Set volume to {} for track: {}",
                volume,
                tracks[track_id].name
            );
        } else {
            // Update track volume even if not playing
            tracks[track_id].volume_level = volume;
            events.push(AudioEvent::VolumeChanged { track_id, volume });
        }

        Ok(events)
    }

    /// Stop all playing tracks
    fn stop_all_tracks(&mut self, tracks: &mut [NoiseTrack]) -> Result<Vec<AudioEvent>, AppError> {
        let mut events = Vec::new();
        let track_ids: Vec<usize> = self.playing_handles.keys().copied().collect();

        for track_id in track_ids {
            events.extend(self.stop_track(track_id, tracks)?);
        }

        self.global_state = PlaybackState::Stopped;
        log::info!("Stopped all tracks");
        Ok(events)
    }

    /// Pause all playing tracks
    fn pause_all_tracks(&mut self, tracks: &mut [NoiseTrack]) -> Result<Vec<AudioEvent>, AppError> {
        let mut events = Vec::new();
        let track_ids: Vec<usize> = self.playing_handles.keys().copied().collect();

        for track_id in track_ids {
            if matches!(self.track_state(track_id), PlaybackState::Playing) {
                events.extend(self.pause_track(track_id, tracks)?);
            }
        }

        self.update_global_state();
        log::info!("Paused all tracks");
        Ok(events)
    }

    /// Resume all paused tracks
    fn resume_all_tracks(
        &mut self,
        tracks: &mut [NoiseTrack],
    ) -> Result<Vec<AudioEvent>, AppError> {
        let mut events = Vec::new();
        let track_ids: Vec<usize> = self.playing_handles.keys().copied().collect();

        for track_id in track_ids {
            if matches!(self.track_state(track_id), PlaybackState::Paused) {
                events.extend(self.resume_track(track_id, tracks)?);
            }
        }

        self.update_global_state();
        log::info!("Resumed all tracks");
        Ok(events)
    }

    /// Update the global playback state based on individual track states
    fn update_global_state(&mut self) {
        if self.playing_handles.is_empty() {
            self.global_state = PlaybackState::Stopped;
            return;
        }

        let mut has_playing = false;
        let mut has_paused = false;

        for handle in self.playing_handles.values() {
            match handle.state() {
                PlaybackState::Playing => has_playing = true,
                PlaybackState::Paused => has_paused = true,
                _ => {}
            }
        }

        self.global_state = if has_playing {
            PlaybackState::Playing
        } else if has_paused {
            PlaybackState::Paused
        } else {
            PlaybackState::Stopped
        };
    }

    /// Create a tween for smooth audio transitions
    fn create_tween(&self) -> Tween {
        Tween {
            duration: self.default_settings.fade_duration,
            easing: kira::Easing::Linear,
            start_time: kira::StartTime::Immediate,
        }
    }

    /// Cleanup finished tracks (should be called periodically)
    pub fn cleanup_finished_tracks(&mut self, tracks: &mut [NoiseTrack]) -> Vec<AudioEvent> {
        let mut events = Vec::new();
        let mut finished_tracks = Vec::new();

        // Find finished tracks
        for (&track_id, handle) in &self.playing_handles {
            if matches!(handle.state(), PlaybackState::Stopped) {
                finished_tracks.push(track_id);
            }
        }

        // Remove finished tracks
        for track_id in finished_tracks {
            self.playing_handles.remove(&track_id);
            if track_id < tracks.len() {
                tracks[track_id].state = PlaybackState::Stopped;
                events.push(AudioEvent::TrackFinished(track_id));
            }
        }

        self.update_global_state();
        events
    }

    /// Get current audio system statistics
    pub fn get_stats(&self) -> AudioStats {
        AudioStats {
            total_tracks: self.playing_handles.len(),
            playing_tracks: self
                .playing_handles
                .values()
                .filter(|h| matches!(h.state(), PlaybackState::Playing))
                .count(),
            paused_tracks: self
                .playing_handles
                .values()
                .filter(|h| matches!(h.state(), PlaybackState::Paused))
                .count(),
            global_state: self.global_state,
            is_initialized: self.is_initialized(),
            latency_ms: None,
            cpu_usage: None,
        }
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            manager: None,
            playing_handles: HashMap::new(),
            global_state: PlaybackState::Stopped,
            default_settings: AudioSettings::default(),
        })
    }
}

impl std::fmt::Debug for AudioSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioSystem")
            .field("manager", &self.manager.is_some())
            .field("playing_handles", &self.playing_handles.len())
            .field("global_state", &self.global_state)
            .field("default_settings", &self.default_settings)
            .finish()
    }
}

/// Convert decibel value to user-friendly percentage (0-100)
/// -60 dB = 0%, 0 dB = 100%
pub fn db_to_percentage(db: f32) -> f32 {
    // Clamp to safe range
    let clamped_db = db.clamp(-60.0, 0.0);
    // Convert to 0-100 scale
    ((clamped_db + 60.0) / 60.0) * 100.0
}

/// Convert user-friendly percentage (0-100) to decibel value
/// 0% = -60 dB, 100% = 0 dB
pub fn percentage_to_db(percentage: f32) -> f32 {
    // Clamp to 0-100 range
    let clamped_percentage = percentage.clamp(0.0, 100.0);
    // Convert to dB scale
    (clamped_percentage / 100.0) * 60.0 - 60.0
}

/// Get a user-friendly volume label for display
pub fn get_volume_label(db: f32) -> String {
    let percentage = db_to_percentage(db);
    if percentage < 1.0 {
        "Muted".to_string()
    } else if percentage < 20.0 {
        "Very Quiet".to_string()
    } else if percentage < 50.0 {
        "Quiet".to_string()
    } else if percentage < 80.0 {
        "Normal".to_string()
    } else {
        "Loud".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_audio_system_creation() {
        let result = AudioSystem::new();
        // Audio system creation might fail in test environment without audio device
        // This is expected behavior
        match result {
            Ok(audio_system) => {
                assert!(audio_system.is_initialized());
                assert_eq!(audio_system.global_state(), PlaybackState::Stopped);
            }
            Err(AppError::Audio(AudioError::InitializationFailed)) => {
                // Expected in test environment
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_audio_settings_default() {
        let settings = AudioSettings::default();
        assert_eq!(settings.default_volume, crate::models::DEFAULT_VOLUME_DB);
        assert_eq!(settings.fade_duration, Duration::from_secs(1));
        assert!(settings.loop_region.is_some());
    }

    #[test]
    fn test_audio_stats() {
        let audio_system = AudioSystem::default();
        let stats = audio_system.get_stats();
        assert_eq!(stats.total_tracks, 0);
        assert_eq!(stats.playing_tracks, 0);
        assert_eq!(stats.paused_tracks, 0);
        assert_eq!(stats.global_state, PlaybackState::Stopped);
    }

    #[test]
    fn test_volume_conversion() {
        // Test dB to percentage conversion
        assert_eq!(db_to_percentage(-60.0), 0.0);
        assert_eq!(db_to_percentage(0.0), 100.0);
        assert_eq!(db_to_percentage(-30.0), 50.0);
        assert_eq!(db_to_percentage(-20.0), 66.66667);

        // Test percentage to dB conversion
        assert_eq!(percentage_to_db(0.0), -60.0);
        assert_eq!(percentage_to_db(100.0), 0.0);
        assert_eq!(percentage_to_db(50.0), -30.0);

        // Test clamping
        assert_eq!(db_to_percentage(-100.0), 0.0); // Should clamp to -60
        assert_eq!(db_to_percentage(20.0), 100.0); // Should clamp to 0
        assert_eq!(percentage_to_db(-10.0), -60.0); // Should clamp to 0
        assert_eq!(percentage_to_db(150.0), 0.0); // Should clamp to 100
    }

    #[test]
    fn test_volume_labels() {
        assert_eq!(get_volume_label(-60.0), "Muted");
        assert_eq!(get_volume_label(-50.0), "Very Quiet");
        assert_eq!(get_volume_label(-30.0), "Normal"); // -30 dB = 50%, which is >= 50%
        assert_eq!(get_volume_label(-15.0), "Normal");
        assert_eq!(get_volume_label(-5.0), "Loud");
    }
}
