use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Card Renderer".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .insert_resource(CardConfig {
            hover_scale: 1.3,
            animation_speed: 5.0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (detect_card_hover, animate_card_scale))
        .run();
}

// Component to mark entities as cards
#[derive(Component)]
struct Card {
    base_scale: Vec3,
    target_scale: Vec3,
    size: Vec2,
}

// Resource for card configuration
#[derive(Resource)]
struct CardConfig {
    hover_scale: f32,
    animation_speed: f32,
}

// Setup system to spawn the card
fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Card dimensions
    let card_width = 200.0;
    let card_height = 300.0;
    let base_scale = Vec3::ONE;

    // Spawn card sprite at center of screen
    commands.spawn((
        Sprite {
            color: Color::srgb(0.9, 0.9, 0.95),
            custom_size: Some(Vec2::new(card_width, card_height)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(base_scale),
        Card {
            base_scale,
            target_scale: base_scale,
            size: Vec2::new(card_width, card_height),
        },
    ));
}

// System to detect if mouse is hovering over the card
fn detect_card_hover(
    mut cards: Query<(&mut Card, &Transform)>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    config: Res<CardConfig>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };

    if let Some(cursor_position) = window.cursor_position() {
        // Convert cursor position to world coordinates
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            for (mut card, transform) in cards.iter_mut() {
                let card_position = transform.translation.truncate();
                let scaled_size = card.size * transform.scale.truncate();

                // Check if cursor is within card bounds
                let half_size = scaled_size / 2.0;
                let is_hovering = world_position.x >= card_position.x - half_size.x
                    && world_position.x <= card_position.x + half_size.x
                    && world_position.y >= card_position.y - half_size.y
                    && world_position.y <= card_position.y + half_size.y;

                // Update target scale based on hover state
                if is_hovering {
                    card.target_scale = card.base_scale * config.hover_scale;
                } else {
                    card.target_scale = card.base_scale;
                }
            }
        }
    } else {
        // If cursor is not in window, reset to base scale
        for (mut card, _) in cards.iter_mut() {
            card.target_scale = card.base_scale;
        }
    }
}

// System to smoothly animate card scale
fn animate_card_scale(
    mut cards: Query<(&Card, &mut Transform)>,
    time: Res<Time>,
    config: Res<CardConfig>,
) {
    for (card, mut transform) in cards.iter_mut() {
        // Smoothly interpolate current scale towards target scale
        transform.scale = transform.scale.lerp(
            card.target_scale,
            time.delta_secs() * config.animation_speed,
        );
    }
}
