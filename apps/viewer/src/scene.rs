//! 3D scene: camera and lighting. The room view (#27) builds on this plugin.

use bevy::prelude::*;

/// Spawns the camera and light that every 3D view shares.
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    // Looks down at the centre of a 4 m x 3 m room from one corner.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 6.0, 7.0).looking_at(Vec3::new(2.0, 0.0, 1.5), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
