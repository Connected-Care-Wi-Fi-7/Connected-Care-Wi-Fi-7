//! Connected Care desktop viewer. See `docs/VIEWER.md` for the plugin layout.
#![forbid(unsafe_code)]
// No console window behind the app in Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod bridge;
mod fake_source;
mod scene;
mod ui;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Connected Care".into(),
                name: Some("Connected Care".into()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            bridge::BridgePlugin,
            fake_source::FakeSourcePlugin,
            scene::ScenePlugin,
            ui::UiPlugin,
        ))
        .run();
}
