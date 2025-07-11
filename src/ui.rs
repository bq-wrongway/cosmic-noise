//! User Interface module for the Cosmic Noise application.
//!
//! This module contains all UI-related functionality including views, components,
//! and styling. It follows the Elm architecture pattern with clear separation
//! between view logic and business logic.

pub mod components;
pub mod styles;
pub mod view;

// Re-export main UI functions for convenient access
pub use components::{error_display, track_card};
pub use styles::card_button_style;
pub use view::main_view;
