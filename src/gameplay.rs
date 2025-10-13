use bevy::prelude::*;
use crate::{GameState, CardConfig, CardData};

// Plugin initializer for gameplay systems
pub fn init_gameplay_systems(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup_gameplay)
        .add_systems(OnExit(GameState::Playing), cleanup_gameplay)
        .add_systems(
            Update,
            (
                card_hover_system,
                card_animation_system,
            )
            .run_if(in_state(GameState::Playing)),
        );
}

// Marker component for game entities
#[derive(Component)]
pub struct GameEntity;

// Card component that holds the card's data
#[derive(Component)]
pub struct Card {
    pub data: CardData,
    pub is_hovered: bool,
    pub target_scale: f32,
    pub base_size: Vec2,
}

impl Card {
    pub fn new(data: CardData, base_size: Vec2) -> Self {
        Self {
            data,
            is_hovered: false,
            target_scale: 1.0,
            base_size,
        }
    }
}

// Component to mark the text child of a card
#[derive(Component)]
pub struct CardText;

// Setup gameplay (spawn initial card)
pub fn setup_gameplay(mut commands: Commands) {
    // Create a sample card
    let card_data = CardData::new("Sample Card");
    let card_size = Vec2::new(200.0, 300.0);
    
    // Spawn the card entity as a sprite with children
    commands.spawn((
        Card::new(card_data.clone(), card_size),
        Sprite {
            color: Color::srgb(0.95, 0.95, 0.98),  // White card background
            custom_size: Some(card_size),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GameEntity,
    ))
    .with_children(|parent| {
        // Card border as a slightly larger sprite behind the card
        parent.spawn((
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.4),  // Border color
                custom_size: Some(card_size + Vec2::splat(6.0)),  // 3px border on each side
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -0.1),  // Slightly behind
        ));
        
        // Card text
        parent.spawn((
            Text2d::new(&card_data.name),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.1, 0.1, 0.15)),  // Dark text
            Transform::from_xyz(0.0, 0.0, 0.1),  // In front of card
            CardText,
        ));
    });
}

// Cleanup gameplay entities
pub fn cleanup_gameplay(mut commands: Commands, game_entities: Query<Entity, With<GameEntity>>) {
    for entity in game_entities.iter() {
        commands.entity(entity).despawn();
    }
}

// System to detect card hover (using mouse position and sprite bounds)
pub fn card_hover_system(
    mut card_query: Query<(&mut Card, &Transform, &Sprite, &Children)>,
    mut sprite_query: Query<&mut Sprite, Without<Card>>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    card_config: Res<CardConfig>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };
    
    let Some((camera, camera_transform)) = camera_query.iter().next() else {
        return;
    };
    
    // Get cursor position in world space
    let cursor_world_pos: Option<Vec2> = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok());
    
    for (mut card, transform, sprite, children) in card_query.iter_mut() {
        let is_hovered = if let (Some(cursor_pos), Some(size)) = (cursor_world_pos, sprite.custom_size) {
            let card_pos = transform.translation.truncate();
            let half_size = size * transform.scale.truncate() / 2.0;
            
            cursor_pos.x >= card_pos.x - half_size.x &&
            cursor_pos.x <= card_pos.x + half_size.x &&
            cursor_pos.y >= card_pos.y - half_size.y &&
            cursor_pos.y <= card_pos.y + half_size.y
        } else {
            false
        };
        
        if is_hovered != card.is_hovered {
            card.is_hovered = is_hovered;
            card.target_scale = if is_hovered {
                card_config.hover_scale
            } else {
                1.0
            };
            
            // Update border color (first child is the border)
            if let Some(&border_entity) = children.get(0) {
                if let Ok(mut border_sprite) = sprite_query.get_mut(border_entity) {
                    border_sprite.color = if is_hovered {
                        Color::srgb(0.4, 0.6, 0.9)  // Blue highlight
                    } else {
                        Color::srgb(0.3, 0.3, 0.4)  // Normal border
                    };
                }
            }
        }
    }
}

// System to animate card scale
pub fn card_animation_system(
    mut card_query: Query<(&Card, &mut Transform)>,
    card_config: Res<CardConfig>,
    time: Res<Time>,
) {
    for (card, mut transform) in card_query.iter_mut() {
        let current_scale = transform.scale.x;
        let scale_diff = card.target_scale - current_scale;
        
        // Smoothly interpolate to target scale
        if scale_diff.abs() > 0.001 {
            let new_scale = current_scale + scale_diff * card_config.animation_speed * time.delta_secs();
            transform.scale = Vec3::splat(new_scale);
        } else {
            transform.scale = Vec3::splat(card.target_scale);
        }
    }
}
