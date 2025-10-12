use bevy::prelude::*;

// Setup camera (runs once at startup)
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
