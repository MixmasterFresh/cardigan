use bevy::prelude::*;
use bevy::app::AppExit;

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
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(OnExit(GameState::Menu), cleanup_menu)
        .add_systems(Update, (menu_button_system, menu_button_interaction).run_if(in_state(GameState::Menu)))
        .add_systems(OnEnter(GameState::Playing), setup_game)
        .add_systems(OnExit(GameState::Playing), cleanup_game)
        .add_systems(Update, (detect_card_hover, animate_card_scale, handle_pause_input).run_if(in_state(GameState::Playing)))
        .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
        .add_systems(OnExit(GameState::Paused), (cleanup_pause_menu, cleanup_game_on_menu_return))
        .add_systems(Update, (pause_button_system, pause_button_interaction, handle_pause_input).run_if(in_state(GameState::Paused)))
        .add_systems(OnEnter(GameState::Options), setup_options)
        .add_systems(OnExit(GameState::Options), cleanup_options)
        .add_systems(Update, (options_button_system, options_button_interaction).run_if(in_state(GameState::Options)))
        .run();
}

// Game states
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    Options,
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

// Marker component for menu entities
#[derive(Component)]
struct MenuEntity;

// Marker component for game entities
#[derive(Component)]
struct GameEntity;

// Marker component for options entities
#[derive(Component)]
struct OptionsEntity;

// Marker component for pause menu entities
#[derive(Component)]
struct PauseEntity;

// Component for menu buttons
#[derive(Component)]
enum MenuButton {
    Play,
    Options,
    Exit,
}

// Component for options buttons
#[derive(Component)]
enum OptionsButton {
    Back,
}

// Component for pause menu buttons
#[derive(Component)]
enum PauseButton {
    Resume,
    MainMenu,
}

// Setup camera (runs once at startup)
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// Setup menu UI
fn setup_menu(mut commands: Commands) {
    // Root node for the menu
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            MenuEntity,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("CARDIGAN"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
            ));

            // Play button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    BorderColor::from(Color::srgb(0.4, 0.4, 0.5)),
                    MenuButton::Play,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("PLAY"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));
                });
            
            // Options button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    BorderColor::from(Color::srgb(0.4, 0.4, 0.5)),
                    MenuButton::Options,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("OPTIONS"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));
                });
            
            // Exit button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    BorderColor::from(Color::srgb(0.4, 0.4, 0.5)),
                    MenuButton::Exit,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("EXIT"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));
                });
        });
}

// Cleanup menu entities
fn cleanup_menu(mut commands: Commands, menu_entities: Query<Entity, With<MenuEntity>>) {
    for entity in menu_entities.iter() {
        commands.entity(entity).despawn();
    }
}

// Handle button interactions (hover effects)
fn menu_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
                *border_color = BorderColor::from(Color::srgb(0.6, 0.6, 0.7));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
                *border_color = BorderColor::from(Color::srgb(0.7, 0.7, 0.8));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::from(Color::srgb(0.4, 0.4, 0.5));
            }
        }
    }
}

// Handle button clicks
fn menu_button_system(
    interaction_query: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                MenuButton::Play => {
                    next_state.set(GameState::Playing);
                }
                MenuButton::Options => {
                    next_state.set(GameState::Options);
                }
                MenuButton::Exit => {
                    exit.write(AppExit::Success);
                }
            }
        }
    }
}

// Setup game (spawn the card)
fn setup_game(mut commands: Commands) {
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
fn cleanup_game(mut commands: Commands, game_entities: Query<Entity, With<GameEntity>>) {
    for entity in game_entities.iter() {
        commands.entity(entity).despawn();
    }
}

// Handle pause input (ESC key)
fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}

// Setup pause menu UI
fn setup_pause_menu(mut commands: Commands) {
    // Root node for the pause menu
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseEntity,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
            ));

            // Resume button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    BorderColor::from(Color::srgb(0.4, 0.4, 0.5)),
                    PauseButton::Resume,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("RESUME"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));
                });

            // Main Menu button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    BorderColor::from(Color::srgb(0.4, 0.4, 0.5)),
                    PauseButton::MainMenu,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("MAIN MENU"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));
                });
        });
}

// Cleanup pause menu entities
fn cleanup_pause_menu(mut commands: Commands, pause_entities: Query<Entity, With<PauseEntity>>) {
    for entity in pause_entities.iter() {
        commands.entity(entity).despawn();
    }
}

// Cleanup game entities when returning to menu from pause
fn cleanup_game_on_menu_return(
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
    next_state: Option<Res<NextState<GameState>>>,
) {
    // Only cleanup if we're transitioning to Menu
    if let Some(next) = next_state {
        if matches!(next.as_ref(), NextState::Pending(GameState::Menu)) {
            for entity in game_entities.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

// Handle pause button interactions (hover effects)
fn pause_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<PauseButton>),
    >,
) {
    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
                *border_color = BorderColor::from(Color::srgb(0.6, 0.6, 0.7));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
                *border_color = BorderColor::from(Color::srgb(0.7, 0.7, 0.8));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::from(Color::srgb(0.4, 0.4, 0.5));
            }
        }
    }
}

// Handle pause button clicks
fn pause_button_system(
    interaction_query: Query<(&Interaction, &PauseButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                PauseButton::Resume => {
                    next_state.set(GameState::Playing);
                }
                PauseButton::MainMenu => {
                    next_state.set(GameState::Menu);
                }
            }
        }
    }
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

// Setup options UI
fn setup_options(mut commands: Commands) {
    // Root node for the options menu
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            OptionsEntity,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("OPTIONS"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
            ));

            // Placeholder text for future options
            parent.spawn((
                Text::new("Settings coming soon..."),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // Back button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    BorderColor::from(Color::srgb(0.4, 0.4, 0.5)),
                    OptionsButton::Back,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("BACK"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.95)),
                    ));
                });
        });
}

// Cleanup options entities
fn cleanup_options(mut commands: Commands, options_entities: Query<Entity, With<OptionsEntity>>) {
    for entity in options_entities.iter() {
        commands.entity(entity).despawn();
    }
}

// Handle options button interactions (hover effects)
fn options_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<OptionsButton>),
    >,
) {
    for (interaction, mut bg_color, mut border_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
                *border_color = BorderColor::from(Color::srgb(0.6, 0.6, 0.7));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
                *border_color = BorderColor::from(Color::srgb(0.7, 0.7, 0.8));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::from(Color::srgb(0.4, 0.4, 0.5));
            }
        }
    }
}

// Handle options button clicks
fn options_button_system(
    interaction_query: Query<(&Interaction, &OptionsButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                OptionsButton::Back => {
                    next_state.set(GameState::Menu);
                }
            }
        }
    }
}
