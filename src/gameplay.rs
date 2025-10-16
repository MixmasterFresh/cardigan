use bevy::prelude::*;
use crate::{GameState, CardConfig, CardData};

// Plugin initializer for gameplay systems
pub fn init_gameplay_systems(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), setup_gameplay)
        .add_systems(OnExit(GameState::Playing), cleanup_gameplay)
        .add_systems(
            Update,
            (
                window_resize_system,      // Handle window resize first
                hand_layout_system,        // Layout first (position, rotation)
                card_hover_system,         // Detect hover
                card_animation_system,     // Animate scale and z-position last
                deck_click_system,
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
    pub target_position: Vec2,  // Target x, y position for smooth movement
}

impl Card {
    pub fn new(data: CardData, base_size: Vec2) -> Self {
        Self {
            data,
            is_hovered: false,
            target_scale: 1.0,
            base_size,
            target_position: Vec2::ZERO,
        }
    }
}

// Component to mark the text child of a card
#[derive(Component)]
pub struct CardText;

// Component to mark cards that are in the player's hand
#[derive(Component)]
pub struct InHand {
    pub hand_index: usize,
}

// Component to mark the deck entity
#[derive(Component)]
pub struct Deck;

// Component to mark the empty deck placeholder
#[derive(Component)]
pub struct DeckEmpty;

// Resource to track remaining cards in deck
#[derive(Resource)]
pub struct DeckCards {
    pub cards: Vec<CardData>,
}

// Resource to track window dimensions for anchoring
#[derive(Resource)]
pub struct WindowDimensions {
    pub width: f32,
    pub height: f32,
}

impl Default for WindowDimensions {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 720.0,
        }
    }
}

// Component to mark entities that should be anchored to window edges
#[derive(Component)]
pub enum AnchorPosition {
    BottomCenter { offset_y: f32 },
    TopRight { offset_x: f32, offset_y: f32 },
    BottomRight { offset_x: f32, offset_y: f32 },
}

// Setup gameplay (spawn deck and initialize hand)
pub fn setup_gameplay(mut commands: Commands, window_query: Query<&Window>) {
    let card_size = Vec2::new(200.0, 300.0);

    // Initialize window dimensions resource
    if let Some(window) = window_query.iter().next() {
        commands.insert_resource(WindowDimensions {
            width: window.width(),
            height: window.height(),
        });
    } else {
        commands.insert_resource(WindowDimensions::default());
    }

    // Initialize deck with 10 cards
    let mut deck_cards = Vec::new();
    for i in 1..=10 {
        deck_cards.push(CardData::new(format!("Card {}", i)));
    }
    commands.insert_resource(DeckCards { cards: deck_cards });

    // Calculate deck position based on window dimensions
    let window_dims = if let Some(window) = window_query.iter().next() {
        WindowDimensions {
            width: window.width(),
            height: window.height(),
        }
    } else {
        WindowDimensions::default()
    };
    
    let deck_offset_x = -120.0;  // 120 pixels from right edge
    let deck_offset_y = 170.0;   // 170 pixels from bottom edge
    let deck_x = (window_dims.width / 2.0) + deck_offset_x;
    let deck_y = -(window_dims.height / 2.0) + deck_offset_y;
    
    // Spawn deck visual at bottom-right of screen
    commands.spawn((
        Deck,
        AnchorPosition::BottomRight {
            offset_x: deck_offset_x,
            offset_y: deck_offset_y,
        },
        Sprite {
            color: Color::srgb(0.8, 0.75, 0.7),  // Card back color
            custom_size: Some(card_size),
            ..default()
        },
        Transform::from_xyz(deck_x, deck_y, 0.0),
        GameEntity,
    ))
    .with_children(|parent| {
        // Deck border
        parent.spawn((
            Sprite {
                color: Color::srgb(0.5, 0.4, 0.35),  // Border color
                custom_size: Some(card_size + Vec2::splat(6.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -0.1),
        ));

        // Deck text
        parent.spawn((
            Text2d::new("DECK"),
            TextFont {
                font_size: 36.0,
                ..default()
            },
            TextColor(Color::srgb(0.2, 0.2, 0.2)),
            Transform::from_xyz(0.0, 0.0, 0.1),
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
// Only allows hovering the topmost card under the cursor
pub fn card_hover_system(
    mut card_query: Query<(Entity, &mut Card, &Transform, &Sprite, &Children)>,
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

    // Find the topmost card under the cursor
    let mut topmost_card: Option<(Entity, f32)> = None;

    if let Some(cursor_pos) = cursor_world_pos {
        for (entity, _card, transform, sprite, _children) in card_query.iter() {
            if let Some(size) = sprite.custom_size {
                let card_pos = transform.translation.truncate();
                let half_size = size * transform.scale.truncate() / 2.0;

                let is_under_cursor = cursor_pos.x >= card_pos.x - half_size.x &&
                    cursor_pos.x <= card_pos.x + half_size.x &&
                    cursor_pos.y >= card_pos.y - half_size.y &&
                    cursor_pos.y <= card_pos.y + half_size.y;

                if is_under_cursor {
                    let z = transform.translation.z;
                    if topmost_card.is_none() || z > topmost_card.unwrap().1 {
                        topmost_card = Some((entity, z));
                    }
                }
            }
        }
    }

    // Update hover state for all cards
    for (entity, mut card, _transform, _sprite, children) in card_query.iter_mut() {
        let should_hover = topmost_card.map(|(e, _)| e == entity).unwrap_or(false);

        if should_hover != card.is_hovered {
            card.is_hovered = should_hover;
            card.target_scale = if should_hover {
                card_config.hover_scale
            } else {
                1.0
            };

            // Update border color (first child is the border)
            if let Some(&border_entity) = children.get(0) {
                if let Ok(mut border_sprite) = sprite_query.get_mut(border_entity) {
                    border_sprite.color = if should_hover {
                        Color::srgb(0.4, 0.6, 0.9)  // Blue highlight
                    } else {
                        Color::srgb(0.3, 0.3, 0.4)  // Normal border
                    };
                }
            }
        }
    }
}

// System to animate card scale, position, and z-position
pub fn card_animation_system(
    mut card_query: Query<(&Card, &mut Transform, Option<&InHand>)>,
    card_config: Res<CardConfig>,
    time: Res<Time>,
    hand_query: Query<&InHand>,
) {
    let hand_count = hand_query.iter().count();

    for (card, mut transform, in_hand) in card_query.iter_mut() {
        // Smoothly interpolate to target scale
        let current_scale = transform.scale.x;
        let scale_diff = card.target_scale - current_scale;
        if scale_diff.abs() > 0.001 {
            let new_scale = current_scale + scale_diff * card_config.animation_speed * time.delta_secs();
            transform.scale = Vec3::splat(new_scale);
        } else {
            transform.scale = Vec3::splat(card.target_scale);
        }

        // Smoothly interpolate x and y positions towards target
        let current_pos = transform.translation.truncate();
        let pos_diff = card.target_position - current_pos;
        if pos_diff.length() > 0.1 {
            let new_pos = current_pos + pos_diff * card_config.animation_speed * time.delta_secs();
            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;
        } else {
            transform.translation.x = card.target_position.x;
            transform.translation.y = card.target_position.y;
        }

        // Update z-position based on hover state for cards in hand
        if let Some(in_hand) = in_hand {
            // Base z is higher for cards on the left (lower index)
            // Use 10.0 increments to ensure clear separation
            let base_z = (hand_count - in_hand.hand_index) as f32 * 10.0;
            // Hovered cards get +100 to be clearly in front
            let target_z = if card.is_hovered {
                base_z + 100.0
            } else {
                base_z
            };

            // Set z-position instantly (no smooth interpolation)
            transform.translation.z = target_z;
        }
    }
}

// System to handle clicking on the deck to draw cards
pub fn deck_click_system(
    mut commands: Commands,
    deck_query: Query<(Entity, &Transform, &Sprite, &Children), With<Deck>>,
    mut deck_cards: ResMut<DeckCards>,
    hand_query: Query<&InHand>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(window) = window_query.iter().next() else {
        return;
    };

    let Some((camera, camera_transform)) = camera_query.iter().next() else {
        return;
    };

    // Get cursor position in world space
    let Some(cursor_world_pos) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok()) else {
        return;
    };

    // Check if deck was clicked
    if let Ok((deck_entity, deck_transform, deck_sprite, deck_children)) = deck_query.single() {
        let Some(deck_size) = deck_sprite.custom_size else {
            return;
        };

        let deck_pos = deck_transform.translation.truncate();
        let half_size = deck_size / 2.0;

        let clicked_deck = cursor_world_pos.x >= deck_pos.x - half_size.x &&
                          cursor_world_pos.x <= deck_pos.x + half_size.x &&
                          cursor_world_pos.y >= deck_pos.y - half_size.y &&
                          cursor_world_pos.y <= deck_pos.y + half_size.y;

        if clicked_deck && !deck_cards.cards.is_empty() {
            // Draw a card from the deck
            let card_data = deck_cards.cards.remove(0);
            let card_size = Vec2::new(200.0, 300.0);
            let hand_index = hand_query.iter().count();
            let hand_count = hand_index + 1;

            // Calculate z position: cards on the left (lower index) should be in front
            // Use larger z increments (10.0 instead of 0.1) to ensure proper layering
            let z = (hand_count - hand_index) as f32 * 10.0;

            // Generate a random color for the card
            use rand::Rng;
            #[allow(deprecated)]
            let mut rng = rand::thread_rng();
            #[allow(deprecated)]
            let card_color = Color::srgb(
                rng.gen_range(0.5..1.0),
                rng.gen_range(0.5..1.0),
                rng.gen_range(0.5..1.0),
            );

            // Spawn the new card
            commands.spawn((
                Card::new(card_data.clone(), card_size),
                InHand { hand_index },
                Sprite {
                    color: card_color,
                    custom_size: Some(card_size),
                    ..default()
                },
                Transform::from_xyz(0.0, -250.0, z),
                GameEntity,
            ))
            .with_children(|parent| {
                // Card border (behind the card)
                parent.spawn((
                    Sprite {
                        color: Color::srgb(0.3, 0.3, 0.4),
                        custom_size: Some(card_size + Vec2::splat(6.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, -1.0),
                ));

                // Card text (in front of the card but still relative to parent)
                parent.spawn((
                    Text2d::new(&card_data.name),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.1, 0.1, 0.15)),
                    Transform::from_xyz(0.0, 0.0, 0.01),
                    CardText,
                ));
            });

            // If deck is now empty, replace with empty deck placeholder
            if deck_cards.cards.is_empty() {
                // Despawn children first
                for child in deck_children.iter() {
                    commands.entity(child).despawn();
                }
                // Then despawn the deck entity
                commands.entity(deck_entity).despawn();

                let deck_pos = deck_transform.translation;
                let card_size = Vec2::new(200.0, 300.0);

                // Spawn empty deck placeholder
                commands.spawn((
                    DeckEmpty,
                    AnchorPosition::BottomRight {
                        offset_x: -120.0,  // 120 pixels from right edge
                        offset_y: 170.0,   // 170 pixels from bottom edge
                    },
                    Sprite {
                        color: Color::NONE,  // Transparent background
                        custom_size: Some(card_size),
                        ..default()
                    },
                    Transform::from_xyz(deck_pos.x, deck_pos.y, 0.0),
                    GameEntity,
                ))
                .with_children(|parent| {
                    // Dotted border (we'll use a solid border with transparency for now)
                    parent.spawn((
                        Sprite {
                            color: Color::srgba(0.3, 0.3, 0.4, 0.5),  // Semi-transparent border
                            custom_size: Some(card_size),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, -0.05),
                    ));

                    // Inner border to create outline effect
                    parent.spawn((
                        Sprite {
                            color: Color::srgba(0.1, 0.1, 0.15, 0.0),  // Transparent inside
                            custom_size: Some(card_size - Vec2::splat(10.0)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, -0.04),
                    ));

                    // "deck" text
                    parent.spawn((
                        Text2d::new("deck"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.3, 0.3, 0.4, 0.6)),
                        Transform::from_xyz(0.0, 0.0, 0.1),
                    ));
                });
            }
        }
    }
}

// System to arrange cards in hand in a splayed arc
pub fn hand_layout_system(
    mut hand_query: Query<(&InHand, &mut Card, &mut Transform)>,
    window_dims: Res<WindowDimensions>,
) {
    let hand_count = hand_query.iter().count();
    if hand_count == 0 {
        return;
    }

    // Hand layout parameters
    let hand_offset_from_bottom = 180.0;  // Distance from bottom edge
    let hand_y = -(window_dims.height / 2.0) + hand_offset_from_bottom;
    let card_spacing = 80.0;  // Horizontal spacing between card centers
    let arc_height = 30.0;  // Height of the arc
    let rotation_per_card = 0.08;  // Rotation in radians per card from center
    let hover_spread = 60.0;  // Additional spacing when card is hovered

    // Find which card is hovered (if any)
    let hovered_index: Option<usize> = hand_query
        .iter()
        .find(|(_, card, _)| card.is_hovered)
        .map(|(in_hand, _, _)| in_hand.hand_index);

    // Calculate total width and starting position
    let total_width = (hand_count - 1) as f32 * card_spacing;
    let start_x = -total_width / 2.0;

    for (in_hand, mut card, mut transform) in hand_query.iter_mut() {
        let index = in_hand.hand_index;

        // Calculate base position along the arc
        let mut x_offset = index as f32 * card_spacing;

        // Apply spreading effect when a card is hovered
        if let Some(hovered_idx) = hovered_index {
            if index > hovered_idx {
                // Cards to the right of hovered card: push right
                x_offset += hover_spread;
            } else if index < hovered_idx {
                // Cards to the left of hovered card: push left
                x_offset -= hover_spread;
            }
        }

        let x = start_x + x_offset;

        // Calculate arc (parabolic curve)
        let center_offset = index as f32 - (hand_count - 1) as f32 / 2.0;
        let normalized_offset = center_offset / ((hand_count as f32) / 2.0).max(1.0);
        let y_offset = arc_height * (1.0 - normalized_offset * normalized_offset);
        let y = hand_y + y_offset;

        // Calculate rotation (cards fan outward)
        let rotation = -center_offset * rotation_per_card;

        // Set target position for smooth interpolation by card_animation_system
        card.target_position = Vec2::new(x, y);

        // Update rotation directly (rotation changes instantly)
        // Note: x, y, and z positions are managed by card_animation_system
        transform.rotation = Quat::from_rotation_z(rotation);
    }
}

// System to handle window resizing and update anchored entities
pub fn window_resize_system(
    mut window_dims: ResMut<WindowDimensions>,
    window_query: Query<&Window>,
    mut anchored_query: Query<(&AnchorPosition, &mut Transform)>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    let new_width = window.width();
    let new_height = window.height();

    // Only update if dimensions actually changed
    if (new_width - window_dims.width).abs() > 0.1 || (new_height - window_dims.height).abs() > 0.1 {
        window_dims.width = new_width;
        window_dims.height = new_height;

        // Update all anchored entities
        for (anchor, mut transform) in anchored_query.iter_mut() {
            match anchor {
                AnchorPosition::BottomCenter { offset_y } => {
                    transform.translation.y = -(new_height / 2.0) + offset_y;
                }
                AnchorPosition::TopRight { offset_x, offset_y } => {
                    transform.translation.x = (new_width / 2.0) + offset_x;
                    transform.translation.y = (new_height / 2.0) + offset_y;
                }
                AnchorPosition::BottomRight { offset_x, offset_y } => {
                    transform.translation.x = (new_width / 2.0) + offset_x;
                    transform.translation.y = -(new_height / 2.0) + offset_y;
                }
            }
        }
    }
}
