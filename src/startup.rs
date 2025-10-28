use bevy::prelude::*;

const WINDOW_HEIGHT: f32 = 1600.0;

// Plugin initializer for startup systems
pub fn init_startup_systems(app: &mut App) {
    app.add_systems(Startup, setup_camera);
}

// Setup camera (runs once at startup)
pub fn setup_camera(mut commands: Commands) {


    commands.spawn((Camera2d, Projection::from(OrthographicProjection {
        scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: WINDOW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
    })));
}
