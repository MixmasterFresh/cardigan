use bevy::prelude::*;
use crate::{GameState, CardConfig, CardData};

// Plugin initializer for gameplay systems
pub fn init_gameplay_systems(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), (setup_gameplay, setup_play_areas).chain())
        .add_systems(OnExit(GameState::Playing), cleanup_gameplay)
        .add_systems(
            Update,
            (
                hand_layout_system,                // Layout first (position, rotation)
                card_hover_system,                 // Detect hover
                card_animation_system,             // Animate scale and z-position last
                deck_click_system,
                card_drag_system,                  // Handle card dragging
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

// Component for cards being dragged
#[derive(Component)]
pub struct Dragging {
    pub offset: Vec2,  // Offset from card center to mouse position
    pub original_zone: CardZone,
}

// Component to mark cards in various zones
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardZone {
    PlayerHand,
    PlayerPlayArea { slot: usize },
    OpponentPlayArea { slot: usize },
    OpponentHand,
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

// Resource to configure play area layout
#[derive(Resource, Clone)]
pub struct PlayAreaConfig {
    pub player_rows: usize,
    pub player_slots_per_row: usize,
    pub opponent_rows: usize,
    pub opponent_slots_per_row: usize,
}

impl Default for PlayAreaConfig {
    fn default() -> Self {
        Self {
            player_rows: 1,
            player_slots_per_row: 5,
            opponent_rows: 1,
            opponent_slots_per_row: 5,
        }
    }
}

// Component to mark card slot placeholders
#[derive(Component)]
pub struct CardSlot {
    pub zone: CardZone,
    pub occupied: bool,
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

// Central gameplay state - source of truth for all game data
#[derive(Resource, Default)]
pub struct GameplayState {
    pub deck: Vec<CardData>,
    pub player_hand: Vec<Entity>,
    pub player_play_area: [Option<Entity>; 5],  // 5 slots, each may contain a card entity
    pub opponent_play_area: [Option<Entity>; 5],
    pub opponent_hand: Vec<Entity>,
}

impl GameplayState {
    pub fn new() -> Self {
        // Initialize deck with 10 cards
        let mut deck = Vec::new();
        for i in 1..=10 {
            deck.push(CardData::new(format!("Card {}", i)));
        }

        Self {
            deck,
            player_hand: Vec::new(),
            player_play_area: [None; 5],
            opponent_play_area: [None; 5],
            opponent_hand: Vec::new(),
        }
    }

    pub fn draw_card(&mut self) -> Option<CardData> {
        self.deck.pop()
    }

    pub fn add_to_hand(&mut self, entity: Entity) {
        self.player_hand.push(entity);
    }

    pub fn remove_from_hand(&mut self, entity: Entity) {
        self.player_hand.retain(|&e| e != entity);
    }

    pub fn play_card_to_slot(&mut self, entity: Entity, slot: usize) -> bool {
        if slot < 5 && self.player_play_area[slot].is_none() {
            self.remove_from_hand(entity);
            self.player_play_area[slot] = Some(entity);
            true
        } else {
            false
        }
    }

    pub fn is_slot_occupied(&self, slot: usize) -> bool {
        slot < 5 && self.player_play_area[slot].is_some()
    }
}

// Resource to track window dimensions for anchoring
#[derive(Resource, Clone)]
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
    TopCenter { offset_y: f32 },
    TopRight { offset_x: f32, offset_y: f32 },
    TopLeft { offset_x: f32, offset_y: f32 },
    BottomRight { offset_x: f32, offset_y: f32 },
    BottomLeft { offset_x: f32, offset_y: f32 },
}

// Layout zones helper - calculates positions for different screen areas
pub struct LayoutZones {
    pub card_size: Vec2,
    pub card_width: f32,
    pub card_height: f32,
}

impl LayoutZones {
    pub fn new(window_dims: &WindowDimensions) -> Self {
        let card_height = window_dims.height * 0.40;
        let card_width = card_height * (2.0 / 3.0);
        let card_size = Vec2::new(card_width, card_height);

        Self {
            card_size,
            card_width,
            card_height,
        }
    }

    /// Get Y position for player's hand (bottom of screen)
    pub fn player_hand_y(&self, window_dims: &WindowDimensions) -> f32 {
        let card_half_height = self.card_height / 2.0;
        let bottom_margin = window_dims.height * 0.025;
        // Bottom of screen is -height/2, add margin and half card height to center card
        -(window_dims.height / 2.0) + bottom_margin + card_half_height
    }

    /// Get Y position for player's play area (above hand, still lower half)
    pub fn player_play_area_y(&self, window_dims: &WindowDimensions) -> f32 {
        let hand_y = self.player_hand_y(window_dims);
        // Player's play area is above the hand
        // Add full card height plus gap (still below center in most cases)
        hand_y + self.card_height + (window_dims.height * 0.08)
    }

    /// Get Y position for opponent's play area (upper half, below opponent's hand)
    pub fn opponent_play_area_y(&self, window_dims: &WindowDimensions) -> f32 {
        let card_half_height = self.card_height / 2.0;
        let top_margin = window_dims.height * 0.025;
        let opponent_hand_y = (window_dims.height / 2.0) - top_margin - card_half_height;
        // Opponent's play area is below their hand
        opponent_hand_y - self.card_height - (window_dims.height * 0.08)
    }

    /// Get Y position for opponent's hand (top of screen)
    pub fn opponent_hand_y(&self, window_dims: &WindowDimensions) -> f32 {
        -self.player_hand_y(window_dims)
    }

    /// Calculate positions for a row of card slots
    pub fn calculate_slot_positions(&self, num_slots: usize, center_y: f32) -> Vec<Vec2> {
        if num_slots == 0 {
            return vec![];
        }

        let spacing = self.card_width * 0.2; // 20% of card width between slots
        let total_width = (num_slots as f32 * self.card_width) + ((num_slots - 1) as f32 * spacing);
        let start_x = -total_width / 2.0;

        (0..num_slots)
            .map(|i| {
                let x = start_x + (i as f32 * (self.card_width + spacing)) + (self.card_width / 2.0);
                Vec2::new(x, center_y)
            })
            .collect()
    }
}

// Setup gameplay (spawn deck and initialize hand)
pub fn setup_gameplay(mut commands: Commands, window_query: Query<&Window>) {
    // Initialize window dimensions resource
    let window_dims = if let Some(window) = window_query.iter().next() {
        let dims = WindowDimensions {
            width: window.width(),
            height: window.height(),
        };
        commands.insert_resource(dims.clone());
        dims
    } else {
        let dims = WindowDimensions::default();
        commands.insert_resource(dims.clone());
        dims
    };

    // Card size: Use viewport height as reference for consistent scaling
    // Card height: 40% of viewport height
    // Card width: 2:3 aspect ratio (width = height * 2/3)
    let card_height = window_dims.height * 0.40;
    let card_width = card_height * (2.0 / 3.0);
    let card_size = Vec2::new(card_width, card_height);

    // Initialize gameplay state
    commands.insert_resource(GameplayState::new());

    // Keep DeckCards for backward compatibility (will migrate fully later)
    commands.insert_resource(DeckCards { cards: Vec::new() });

    // Deck position: Scale with viewport height for consistency
    // Offset from right: 1.5x card width, offset from bottom: 0.6x card height
    let deck_offset = card_width * 0.1;
    let deck_offset_x = -(card_width * 0.5 + deck_offset);
    let deck_offset_y = card_height * 0.5 + deck_offset;
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

// Setup play area card slots
pub fn setup_play_areas(
    mut commands: Commands,
    window_query: Query<&Window>,
) {
    let window_dims = if let Some(window) = window_query.iter().next() {
        WindowDimensions {
            width: window.width(),
            height: window.height(),
        }
    } else {
        WindowDimensions::default()
    };

    // Initialize play area configuration - 1 row of 5 slots for now
    let config = PlayAreaConfig {
        player_rows: 1,
        player_slots_per_row: 5,
        opponent_rows: 1,
        opponent_slots_per_row: 5,
    };
    commands.insert_resource(config.clone());

    let layout = LayoutZones::new(&window_dims);

    // Spawn player play area slots
    let player_y = layout.player_play_area_y(&window_dims);
    let player_positions = layout.calculate_slot_positions(config.player_slots_per_row, player_y);

    for (slot_index, position) in player_positions.iter().enumerate() {
        spawn_card_slot(
            &mut commands,
            CardZone::PlayerPlayArea { slot: slot_index },
            *position,
            layout.card_size,
            false,  // Initially unoccupied
        );
    }

    // Spawn opponent play area slots
    let opponent_y = layout.opponent_play_area_y(&window_dims);
    let opponent_positions = layout.calculate_slot_positions(config.opponent_slots_per_row, opponent_y);

    for (slot_index, position) in opponent_positions.iter().enumerate() {
        spawn_card_slot(
            &mut commands,
            CardZone::OpponentPlayArea { slot: slot_index },
            *position,
            layout.card_size,
            false,  // Initially unoccupied
        );
    }
}

// Helper function to spawn a card slot placeholder
fn spawn_card_slot(
    commands: &mut Commands,
    zone: CardZone,
    position: Vec2,
    card_size: Vec2,
    occupied: bool,
) {
    commands.spawn((
        CardSlot {
            zone,
            occupied,
        },
        Sprite {
            color: Color::NONE,
            custom_size: Some(card_size),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, -10.0),
        GameEntity,
    ))
    .with_children(|parent| {
        // Dashed border effect using multiple rectangles
        let border_width = 4.0;
        let dash_length = 20.0;
        let gap_length = 10.0;

        let border_color = Color::srgba(0.4, 0.4, 0.5, 0.4);

        // Top border dashes
        let mut x = -card_size.x / 2.0 + dash_length / 2.0;
        let y_top = card_size.y / 2.0;
        while x + dash_length / 2.0 <= card_size.x / 2.0 {
            let actual_dash_length = (dash_length).min(card_size.x / 2.0 - x + dash_length / 2.0);
            parent.spawn((
                Sprite {
                    color: border_color,
                    custom_size: Some(Vec2::new(actual_dash_length, border_width)),
                    ..default()
                },
                Transform::from_xyz(x, y_top, 0.1),
            ));
            x += dash_length + gap_length;
        }

        // Bottom border dashes
        let mut x = -card_size.x / 2.0 + dash_length / 2.0;
        let y_bottom = -card_size.y / 2.0;
        while x + dash_length / 2.0 <= card_size.x / 2.0 {
            let actual_dash_length = (dash_length).min(card_size.x / 2.0 - x + dash_length / 2.0);
            parent.spawn((
                Sprite {
                    color: border_color,
                    custom_size: Some(Vec2::new(actual_dash_length, border_width)),
                    ..default()
                },
                Transform::from_xyz(x, y_bottom, 0.1),
            ));
            x += dash_length + gap_length;
        }

        // Left border dashes
        let x_left = -card_size.x / 2.0;
        let mut y = -card_size.y / 2.0 + dash_length / 2.0;
        while y + dash_length / 2.0 <= card_size.y / 2.0 {
            let actual_dash_length = (dash_length).min(card_size.y / 2.0 - y + dash_length / 2.0);
            parent.spawn((
                Sprite {
                    color: border_color,
                    custom_size: Some(Vec2::new(border_width, actual_dash_length)),
                    ..default()
                },
                Transform::from_xyz(x_left, y, 0.1),
            ));
            y += dash_length + gap_length;
        }

        // Right border dashes
        let x_right = card_size.x / 2.0;
        let mut y = -card_size.y / 2.0 + dash_length / 2.0;
        while y + dash_length / 2.0 <= card_size.y / 2.0 {
            let actual_dash_length = (dash_length).min(card_size.y / 2.0 - y + dash_length / 2.0);
            parent.spawn((
                Sprite {
                    color: border_color,
                    custom_size: Some(Vec2::new(border_width, actual_dash_length)),
                    ..default()
                },
                Transform::from_xyz(x_right, y, 0.1),
            ));
            y += dash_length + gap_length;
        }
    });
}

// System to detect card hover (using mouse position and sprite bounds)
// Only allows hovering the topmost card under the cursor
pub fn card_hover_system(
    mut card_query: Query<(Entity, &mut Card, &Transform, &Sprite, &Children, Option<&Dragging>)>,
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
        for (entity, _card, transform, sprite, _children, dragging) in card_query.iter() {
            // Skip cards that are being dragged
            if dragging.is_some() {
                continue;
            }

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
    for (entity, mut card, _transform, _sprite, children, dragging) in card_query.iter_mut() {
        // Don't update hover state for dragging cards
        if dragging.is_some() {
            continue;
        }
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

// System to handle card dragging and dropping
pub fn card_drag_system(
    mut commands: Commands,
    mut card_query: Query<(Entity, &mut Card, &Transform, &Sprite, Option<&Dragging>, Option<&InHand>)>,
    mut slot_query: Query<(Entity, &mut CardSlot, &Transform, &Sprite)>,
    mut gameplay_state: ResMut<GameplayState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    let Some((camera, camera_transform)) = camera_query.iter().next() else {
        return;
    };

    let cursor_world_pos = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok());

    // Start dragging
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = cursor_world_pos {
            // Find the topmost card under cursor that's in hand
            let mut topmost_card: Option<(Entity, f32, Vec2)> = None;

            for (entity, _card, transform, sprite, dragging, in_hand) in card_query.iter() {
                // Only allow dragging cards in hand
                if dragging.is_some() || in_hand.is_none() {
                    continue;
                }

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
                            topmost_card = Some((entity, z, card_pos));
                        }
                    }
                }
            }

            // Start dragging the topmost card
            if let Some((entity, _, card_pos)) = topmost_card {
                let offset = cursor_pos - card_pos;
                commands.entity(entity).insert(Dragging {
                    offset,
                    original_zone: CardZone::PlayerHand,
                });
            }
        }
    }

    // Update dragging cards position
    if let Some(cursor_pos) = cursor_world_pos {
        for (entity, mut card, transform, _sprite, dragging, _in_hand) in card_query.iter_mut() {
            if let Some(drag) = dragging {
                let new_pos = cursor_pos - drag.offset;
                card.target_position = new_pos;

                // Bring dragged card to front
                commands.entity(entity).insert(Transform {
                    translation: Vec3::new(new_pos.x, new_pos.y, 1000.0),
                    ..*transform
                });
            }
        }
    }

    // Stop dragging and check for drop
    if mouse_button.just_released(MouseButton::Left) {
        for (entity, mut card, transform, _sprite, dragging, in_hand) in card_query.iter_mut() {
            if let Some(_drag) = dragging {
                let card_pos = transform.translation.truncate();

                // Check if dropped on a valid slot
                let mut dropped_on_slot = false;
                let mut target_slot_entity: Option<Entity> = None;
                let mut target_slot_pos: Option<Vec2> = None;
                let mut target_zone: Option<CardZone> = None;

                for (slot_entity, slot, slot_transform, slot_sprite) in slot_query.iter() {
                    // Only check player play area slots
                    let slot_index = if let CardZone::PlayerPlayArea { slot } = slot.zone {
                        slot
                    } else {
                        continue;
                    };

                    // Check if slot is occupied in gameplay state
                    if gameplay_state.is_slot_occupied(slot_index) {
                        continue;
                    }

                    if let Some(slot_size) = slot_sprite.custom_size {
                        let slot_pos = slot_transform.translation.truncate();
                        let half_size = slot_size / 2.0;

                        let is_over_slot = card_pos.x >= slot_pos.x - half_size.x &&
                            card_pos.x <= slot_pos.x + half_size.x &&
                            card_pos.y >= slot_pos.y - half_size.y &&
                            card_pos.y <= slot_pos.y + half_size.y;

                        if is_over_slot {
                            // Store info for dropping the card
                            target_slot_entity = Some(slot_entity);
                            target_slot_pos = Some(slot_pos);
                            target_zone = Some(slot.zone);
                            dropped_on_slot = true;
                            break;
                        }
                    }
                }

                // Update card and slot if dropped on valid slot
                if dropped_on_slot {
                    if let (Some(slot_entity), Some(slot_pos), Some(zone)) =
                        (target_slot_entity, target_slot_pos, target_zone) {

                        // Extract slot number from zone
                        if let CardZone::PlayerPlayArea { slot: slot_index } = zone {
                            // Remove from hand in gameplay state
                            gameplay_state.remove_from_hand(entity);

                            // Play card to slot in gameplay state
                            gameplay_state.play_card_to_slot(entity, slot_index);

                            // Update card
                            card.target_position = slot_pos;

                            // Remove from hand component
                            if in_hand.is_some() {
                                commands.entity(entity).remove::<InHand>();
                            }

                            // Reset rotation to 0 for played cards
                            commands.entity(entity).insert(Transform {
                                rotation: Quat::IDENTITY,
                                ..Default::default()
                            });

                            // Add zone marker
                            commands.entity(entity).insert(zone);

                            // Mark slot as occupied (for visual consistency)
                            if let Ok((_, mut slot, _, _)) = slot_query.get_mut(slot_entity) {
                                slot.occupied = true;
                            }
                        }
                    }
                }

                // If not dropped on a valid slot, return to hand
                if !dropped_on_slot {
                    // Card will be repositioned by hand_layout_system
                }

                // Remove dragging component
                commands.entity(entity).remove::<Dragging>();
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
            // Protect against underflow when hand_index >= hand_count
            // This can happen temporarily when a card is being removed from hand
            if in_hand.hand_index < hand_count {
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
        } else {
            // Cards not in hand should be at z=0 (play area, etc.)
            transform.translation.z = 0.0;
        }
    }
}

// System to handle clicking on the deck to draw cards
pub fn deck_click_system(
    mut commands: Commands,
    deck_query: Query<(Entity, &Transform, &Sprite, &Children), With<Deck>>,
    mut gameplay_state: ResMut<GameplayState>,
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

        if clicked_deck {
            // Try to draw a card from the gameplay state
            if let Some(card_data) = gameplay_state.draw_card() {

            // Card size: Use viewport height as reference for consistent scaling
            // Card height: 40% of viewport height, width: 2:3 aspect ratio
            let card_height = window.height() * 0.40;
            let card_width = card_height * (2.0 / 3.0);
            let card_size = Vec2::new(card_width, card_height);

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
            let card_entity = commands.spawn((
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
            })
            .id(); // Get the entity ID

            // Add the new card entity to gameplay state
            gameplay_state.add_to_hand(card_entity);
        }

        // If deck is now empty, replace with empty deck placeholder
        if gameplay_state.deck.is_empty() {
                // Despawn children first
                for child in deck_children.iter() {
                    commands.entity(child).despawn();
                }
                // Then despawn the deck entity
                commands.entity(deck_entity).despawn();

                let deck_pos = deck_transform.translation;

                // Card size: Use viewport height as reference for consistent scaling
                let card_height = window.height() * 0.40;
                let card_width = card_height * (2.0 / 3.0);
                let card_size = Vec2::new(card_width, card_height);

                // Deck position: Scale with card size
                let deck_offset = card_width * 0.1;
                let deck_offset_x = -(card_width * 0.5 + deck_offset);
                let deck_offset_y = card_height * 0.5 + deck_offset;

                // Spawn empty deck placeholder
                commands.spawn((
                    DeckEmpty,
                    AnchorPosition::BottomRight {
                        offset_x: deck_offset_x,
                        offset_y: deck_offset_y,
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
    mut hand_query: Query<(&InHand, &mut Card, &mut Transform, Option<&Dragging>)>,
    window_dims: Res<WindowDimensions>,
) {
    let hand_count = hand_query.iter().count();
    if hand_count == 0 {
        return;
    }

    // Hand layout parameters: Scale everything relative to viewport height for consistency
    let card_height = window_dims.height * 0.40;
    let card_width = card_height * (2.0 / 3.0);

    // Spacing based on card width for proportional layout
    let card_spacing = card_width * 0.4;  // 40% of card width between cards
    let arc_height = card_height * 0.1;   // 10% of card height for arc
    let rotation_per_card = 0.08;         // Rotation in radians per card from center
    let hover_spread = card_width * 0.3;  // 30% of card width for hover spread

    // Calculate hand position so the BOTTOM of cards stays at consistent distance from bottom
    // We want the bottom of the lowest card to be 2.5% of viewport height from bottom
    let card_half_height = card_height / 2.0;
    let bottom_margin = window_dims.height * 0.025;  // 2.5% from bottom
    let hand_y = -(window_dims.height / 2.0) + bottom_margin + card_half_height;

    // Find which card is hovered (if any)
    let hovered_index: Option<usize> = hand_query
        .iter()
        .find(|(_, card, _, dragging)| card.is_hovered && dragging.is_none())
        .map(|(in_hand, _, _, _)| in_hand.hand_index);

    // Calculate total width and starting position
    let total_width = (hand_count - 1) as f32 * card_spacing;
    let start_x = -total_width / 2.0;

    for (in_hand, mut card, mut transform, dragging) in hand_query.iter_mut() {
        // Skip cards that are being dragged
        if dragging.is_some() {
            continue;
        }

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
