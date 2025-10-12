use bevy::prelude::*;

// Component to mark entities as cards
#[derive(Component)]
pub struct Card {
    pub base_scale: Vec3,
    pub target_scale: Vec3,
    pub size: Vec2,
}

// Resource for card configuration
#[derive(Resource)]
pub struct CardConfig {
    pub hover_scale: f32,
    pub animation_speed: f32,
}

// Marker component for game entities
#[derive(Component)]
pub struct GameEntity;

// Setup game (spawn the card)
pub fn setup_game(mut commands: Commands) {
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
        GameEntity,
    ));
}

// Cleanup game entities
pub fn cleanup_game(mut commands: Commands, game_entities: Query<Entity, With<GameEntity>>) {
    for entity in game_entities.iter() {
        commands.entity(entity).despawn();
    }
}

// System to detect if mouse is hovering over the card
pub fn detect_card_hover(
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
pub fn animate_card_scale(
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
