// SPDX-License-Identifier: GPL-3.0-only

use app::CosmicNoise;
/// The `app` module is used by convention to indicate the main component of our application.
mod app;
mod config;
mod i18n;
mod utils;

/// The `cosmic::app::run()` function is the starting point of your application.
/// It takes two arguments:
/// - `settings` is a structure that contains everything relevant with your app's configuration, such as antialiasing, themes, icons, etc...
/// - `()` is the flags that your app needs to use before it starts.
///  If your app does not need any flags, you can pass in `()`.
fn main() -> cosmic::iced::Result {
    // std::env::set_var("RUST_LOG", "warn");
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    env_logger::init();
    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    //in case i decide to change this to full fledget app
    // let settings = cosmic::app::Settings::default();
    cosmic::applet::run::<CosmicNoise>(())
}
