//! Connected Care desktop viewer, split into Bevy plugins.
#![forbid(unsafe_code)]
// No console window behind the app in Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[expect(
    dead_code,
    reason = "the fake source and UI that use the bridge land in the next PR (#26 part 2)"
)]
mod bridge;
mod scene;

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
        .add_plugins((bridge::BridgePlugin, scene::ScenePlugin))
        .run();
}
