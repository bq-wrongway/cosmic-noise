// SPDX-License-Identifier: GPL-3.0-only

use app::CosmicNoise;
mod app;
mod core;
mod files;

fn main() -> cosmic::iced::Result {
    let settings = cosmic::app::Settings::default();
    cosmic::app::run::<CosmicNoise>(settings, ())
}
