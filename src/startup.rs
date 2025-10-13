use bevy::prelude::*;

// Plugin initializer for startup systems
pub fn init_startup_systems(app: &mut App) {
    app.add_systems(Startup, setup_camera);
}

// Setup camera (runs once at startup)
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
